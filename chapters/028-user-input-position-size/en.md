# Day 28: User Input — Trader Enters Position Size

## Trading Analogy

Imagine a trading terminal. Before opening a position, the trader must **enter data**: position size, entry price, stop-loss. The program can't guess these values — it must **ask** the user and **read** their response.

In Rust, we use the `std::io` module for this — it allows the program to communicate with the user through the console.

## Basic Input: Reading a Line

```rust
use std::io;

fn main() {
    println!("Enter position size (in lots):");

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    println!("You entered: {}", input.trim());
}
```

**Let's break it down:**

1. `use std::io` — import the input-output module
2. `let mut input = String::new()` — create a mutable empty string
3. `io::stdin()` — get standard input (keyboard)
4. `.read_line(&mut input)` — read a line into our variable
5. `.expect("...")` — handle possible errors
6. `input.trim()` — remove the newline character `\n`

## Why mut and &mut?

```rust
use std::io;

fn main() {
    // mut — because read_line MODIFIES this string
    let mut position_size = String::new();

    println!("Position size:");

    // &mut — we pass a mutable reference
    // read_line writes data into our variable
    io::stdin()
        .read_line(&mut position_size)
        .expect("Failed to read");

    println!("Position: {} lots", position_size.trim());
}
```

**Analogy:** Imagine giving an analyst an empty form (`String::new()`) and saying: "Write the position size here". They **modify** your form — that's why we need `mut`.

## Reading Multiple Values

```rust
use std::io;

fn main() {
    let mut ticker = String::new();
    let mut quantity = String::new();
    let mut price = String::new();

    println!("=== NEW ORDER ===");

    println!("Ticker (e.g., AAPL):");
    io::stdin().read_line(&mut ticker).expect("Error");

    println!("Number of shares:");
    io::stdin().read_line(&mut quantity).expect("Error");

    println!("Entry price:");
    io::stdin().read_line(&mut price).expect("Error");

    println!("\n=== CONFIRMATION ===");
    println!("Ticker: {}", ticker.trim());
    println!("Quantity: {}", quantity.trim());
    println!("Price: ${}", price.trim());
}
```

## Input with Prompt

```rust
use std::io::{self, Write};

fn main() {
    print!("Enter ticker: ");
    io::stdout().flush().unwrap(); // Force output immediately

    let mut ticker = String::new();
    io::stdin().read_line(&mut ticker).expect("Error");

    println!("You selected: {}", ticker.trim().to_uppercase());
}
```

**Why flush()?** `print!` doesn't add a newline, and text might not display immediately. `flush()` forces the buffer to output right away.

## Helper Function for Reading Input

```rust
use std::io::{self, Write};

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    input.trim().to_string()
}

fn main() {
    let ticker = read_input("Ticker: ");
    let quantity = read_input("Quantity: ");
    let entry_price = read_input("Entry price: ");
    let stop_loss = read_input("Stop-loss: ");

    println!("\n╔═══════════════════════════════╗");
    println!("║       ORDER PARAMETERS        ║");
    println!("╠═══════════════════════════════╣");
    println!("║ Ticker:      {:>16} ║", ticker);
    println!("║ Quantity:    {:>16} ║", quantity);
    println!("║ Entry price: ${:>15} ║", entry_price);
    println!("║ Stop-loss:   ${:>15} ║", stop_loss);
    println!("╚═══════════════════════════════╝");
}
```

## Action Selection Menu

```rust
use std::io::{self, Write};

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Error");
    input.trim().to_string()
}

fn main() {
    println!("╔═══════════════════════════════╗");
    println!("║      TRADING TERMINAL         ║");
    println!("╠═══════════════════════════════╣");
    println!("║  1. Open position             ║");
    println!("║  2. Close position            ║");
    println!("║  3. Show portfolio            ║");
    println!("║  4. Trade history             ║");
    println!("║  5. Exit                      ║");
    println!("╚═══════════════════════════════╝");

    let choice = read_input("\nSelect action (1-5): ");

    match choice.as_str() {
        "1" => println!("Opening new position..."),
        "2" => println!("Closing position..."),
        "3" => println!("Loading portfolio..."),
        "4" => println!("Loading history..."),
        "5" => println!("Goodbye!"),
        _ => println!("Unknown command: {}", choice),
    }
}
```

## Entering Trade Direction

```rust
use std::io::{self, Write};

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Error");
    input.trim().to_string()
}

fn main() {
    println!("=== NEW TRADE ===\n");

    let direction = read_input("Direction (buy/sell): ");
    let ticker = read_input("Ticker: ");
    let quantity = read_input("Quantity: ");

    let direction_upper = direction.to_uppercase();
    let is_buy = direction_upper == "BUY" || direction_upper == "B";

    if is_buy {
        println!("\n🟢 BUY {} x {}", quantity, ticker.to_uppercase());
    } else {
        println!("\n🔴 SELL {} x {}", quantity, ticker.to_uppercase());
    }
}
```

## Input Loop Until Valid Value

```rust
use std::io::{self, Write};

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Error");
    input.trim().to_string()
}

fn main() {
    loop {
        let risk = read_input("Risk per trade (1-5%): ");

        // Simple check — is it a number from 1 to 5
        match risk.as_str() {
            "1" | "2" | "3" | "4" | "5" => {
                println!("Risk set to: {}%", risk);
                break;
            }
            _ => {
                println!("Error! Enter a number from 1 to 5");
            }
        }
    }

    println!("Setup complete.");
}
```

## Interactive Position Monitoring

```rust
use std::io::{self, Write};

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Error");
    input.trim().to_string()
}

fn main() {
    let entry_price = 42000.0_f64;
    let position_size = 0.5_f64;

    println!("╔═══════════════════════════════════╗");
    println!("║    BTC POSITION MONITORING        ║");
    println!("╠═══════════════════════════════════╣");
    println!("║  Entry price: ${:.2}            ║", entry_price);
    println!("║  Size: {} BTC                     ║", position_size);
    println!("╚═══════════════════════════════════╝");

    loop {
        let current = read_input("\nCurrent price (or 'exit'): ");

        if current.to_lowercase() == "exit" {
            println!("Exiting monitoring.");
            break;
        }

        // For now, just output as string
        // (number parsing will be in the next chapter)
        println!("Entered price: ${}", current);
        println!("(PnL calculation will be in the next chapter)");
    }
}
```

## Key Points to Remember

| Aspect | Description |
|--------|-------------|
| `String::new()` | Creates an empty string for input |
| `mut` | Variable must be mutable |
| `&mut` | Pass mutable reference to read_line |
| `trim()` | Removes `\n` at end of string |
| `flush()` | Outputs print! buffer immediately |
| `expect()` | Panics if reading error occurs |

## Handling Result Without Panic

```rust
use std::io::{self, Write};

fn read_input_safe(prompt: &str) -> Result<String, io::Error> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

fn main() {
    match read_input_safe("Enter ticker: ") {
        Ok(ticker) => println!("Ticker: {}", ticker),
        Err(e) => println!("Input error: {}", e),
    }
}
```

## Practical Example: Order Form

```rust
use std::io::{self, Write};

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Error");
    input.trim().to_string()
}

fn confirm(prompt: &str) -> bool {
    let answer = read_input(prompt);
    answer.to_lowercase() == "y" || answer.to_lowercase() == "yes"
}

fn main() {
    println!("\n╔═══════════════════════════════════╗");
    println!("║        NEW ORDER FORM             ║");
    println!("╚═══════════════════════════════════╝\n");

    let order_type = read_input("Order type (market/limit): ");
    let side = read_input("Side (buy/sell): ");
    let ticker = read_input("Ticker: ");
    let quantity = read_input("Quantity: ");

    let price = if order_type.to_lowercase() == "limit" {
        read_input("Limit price: ")
    } else {
        String::from("market")
    };

    println!("\n═══════════════════════════════════");
    println!("         REVIEW YOUR ORDER         ");
    println!("═══════════════════════════════════");
    println!("Type:      {}", order_type.to_uppercase());
    println!("Side:      {}", side.to_uppercase());
    println!("Ticker:    {}", ticker.to_uppercase());
    println!("Quantity:  {}", quantity);
    println!("Price:     {}", price);
    println!("═══════════════════════════════════\n");

    if confirm("Confirm order? (y/n): ") {
        println!("\n✓ Order submitted!");
    } else {
        println!("\n✗ Order cancelled.");
    }
}
```

## What We Learned

1. **std::io** — module for input-output operations
2. **stdin().read_line()** — reads a line from user
3. **String::new()** — creates an empty mutable string
4. **trim()** — removes whitespace and newline
5. **flush()** — forces print! buffer to output

## Homework

1. Write a position size calculator: user enters balance, risk percentage, and distance to stop-loss. Program outputs position size (as string for now).

2. Create an interactive trading bot menu with five options and invalid input handling.

3. Implement an order parameter form (ticker, quantity, price, stop-loss, take-profit) with formatted output.

4. Make a quiz program: ask user 5 trading questions and count correct answers.

## Navigation

[← Previous day](../027-shadowing-price-update/en.md) | [Next day →](../029-string-parsing/en.md)
