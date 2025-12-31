# Day 314: FFI: C Library Integration

## Trading Analogy

Imagine you're developing a trading platform in Rust, but there's a powerful technical analysis library written in C — for example, TA-Lib (Technical Analysis Library). It has hundreds of indicators, optimized and tested over the years.

Instead of rewriting everything from scratch in Rust, you can use **FFI (Foreign Function Interface)** — it's like having a translator between two languages:

- **Rust** — your main language, safe and fast
- **C library** — time-tested algorithms
- **FFI** — the bridge between them

This is similar to how large banks integrate legacy trading systems (written in C/C++) with new services. Instead of a complete rewrite, they create interfaces for interaction.

## What is FFI?

**FFI (Foreign Function Interface)** is a mechanism that allows Rust to call functions written in other languages (primarily C), and vice versa.

### Why is this important in trading?

| Scenario | Reason for Using FFI |
|----------|----------------------|
| **Legacy code** | Existing trading systems in C/C++ |
| **Optimized libraries** | TA-Lib, QuickFIX (FIX protocol), libta |
| **Platform APIs** | System calls, exchange gateway drivers |
| **Performance** | Critical sections in low-level C |
| **Integration** | Connecting to broker APIs in C |

## FFI Basics: Calling C Functions from Rust

### Simple Example: Calling a Function from libc

```rust
use std::ffi::CString;
use std::os::raw::c_char;

// Declare an external function from libc
extern "C" {
    fn strlen(s: *const c_char) -> usize;
}

fn main() {
    // Create a C-compatible string
    let ticker = CString::new("BTCUSD").expect("CString creation failed");

    // Call C function (unsafe!)
    unsafe {
        let len = strlen(ticker.as_ptr());
        println!("Ticker '{}' length: {}", ticker.to_str().unwrap(), len);
    }
}
```

### Why `unsafe`?

Rust cannot guarantee the safety of C code:
- **No array bounds checking**
- **Possible null pointers**
- **No lifetime control**
- **Possible memory corruption**

Therefore, all FFI calls require an `unsafe` block.

## Example: Integration with a Simple C Library for SMA Calculation

Imagine we have a C library for calculating Simple Moving Average:

### C Code (sma.h and sma.c)

```c
// sma.h
#ifndef SMA_H
#define SMA_H

#include <stddef.h>

// Calculate Simple Moving Average
double calculate_sma(const double* prices, size_t length, size_t period);

#endif // SMA_H
```

```c
// sma.c
#include "sma.h"

double calculate_sma(const double* prices, size_t length, size_t period) {
    if (length < period || period == 0) {
        return 0.0;
    }

    double sum = 0.0;
    for (size_t i = length - period; i < length; i++) {
        sum += prices[i];
    }

    return sum / period;
}
```

### Rust Integration

```rust
use std::os::raw::c_double;

// Declare the external C function
extern "C" {
    fn calculate_sma(prices: *const c_double, length: usize, period: usize) -> c_double;
}

/// Safe wrapper over the C function
pub fn sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }

    unsafe {
        let result = calculate_sma(prices.as_ptr(), prices.len(), period);
        Some(result)
    }
}

fn main() {
    let btc_prices = vec![
        42000.0, 42500.0, 41800.0, 43200.0, 43500.0,
        44000.0, 43800.0, 44500.0, 45000.0, 44800.0,
    ];

    if let Some(sma_5) = sma(&btc_prices, 5) {
        println!("SMA(5) for BTC: ${:.2}", sma_5);
    }

    if let Some(sma_10) = sma(&btc_prices, 10) {
        println!("SMA(10) for BTC: ${:.2}", sma_10);
    }
}
```

## FFI Data Types: Mapping Between Rust and C

| Rust | C | Description |
|------|---|-------------|
| `i8` | `int8_t` | 8-bit signed integer |
| `u8` | `uint8_t` | 8-bit unsigned integer |
| `i32` | `int32_t` | 32-bit signed integer |
| `f64` | `double` | Double precision float |
| `*const T` | `const T*` | Constant pointer |
| `*mut T` | `T*` | Mutable pointer |
| `()` | `void` | Absence of value |

### Example: Data Structures

```rust
use std::os::raw::{c_char, c_double, c_int};

// Rust structure compatible with C
#[repr(C)]
pub struct Candle {
    timestamp: i64,
    open: c_double,
    high: c_double,
    low: c_double,
    close: c_double,
    volume: c_double,
}

// C function for processing a candle
extern "C" {
    fn analyze_candle(candle: *const Candle) -> c_int;
}

impl Candle {
    fn new(timestamp: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Candle { timestamp, open, high, low, close, volume }
    }

    fn analyze(&self) -> i32 {
        unsafe {
            analyze_candle(self as *const Candle)
        }
    }
}

fn main() {
    let candle = Candle::new(
        1672531200,  // Unix timestamp
        42000.0,     // Open
        43000.0,     // High
        41500.0,     // Low
        42800.0,     // Close
        1500.0,      // Volume
    );

    let signal = candle.analyze();
    match signal {
        1 => println!("📈 Buy signal"),
        -1 => println!("📉 Sell signal"),
        _ => println!("⏸️  Hold position"),
    }
}
```

## Advanced Example: TA-Lib Integration

TA-Lib is a popular technical analysis library in C. Here's how to use it in Rust:

### TA-Lib Function Declarations

```rust
use std::os::raw::{c_double, c_int};

#[repr(C)]
pub enum MAType {
    SMA = 0,   // Simple Moving Average
    EMA = 1,   // Exponential Moving Average
    WMA = 2,   // Weighted Moving Average
    DEMA = 3,  // Double Exponential Moving Average
    TEMA = 4,  // Triple Exponential Moving Average
}

extern "C" {
    // RSI - Relative Strength Index
    fn TA_RSI(
        start_idx: c_int,
        end_idx: c_int,
        in_real: *const c_double,
        opt_in_time_period: c_int,
        out_begin_idx: *mut c_int,
        out_nb_element: *mut c_int,
        out_real: *mut c_double,
    ) -> c_int;

    // MACD - Moving Average Convergence/Divergence
    fn TA_MACD(
        start_idx: c_int,
        end_idx: c_int,
        in_real: *const c_double,
        opt_in_fast_period: c_int,
        opt_in_slow_period: c_int,
        opt_in_signal_period: c_int,
        out_begin_idx: *mut c_int,
        out_nb_element: *mut c_int,
        out_macd: *mut c_double,
        out_signal: *mut c_double,
        out_hist: *mut c_double,
    ) -> c_int;
}

/// Safe wrapper for RSI
pub fn calculate_rsi(prices: &[f64], period: usize) -> Result<Vec<f64>, String> {
    if prices.len() < period {
        return Err("Insufficient data for RSI calculation".to_string());
    }

    let mut out_begin = 0i32;
    let mut out_size = 0i32;
    let mut output = vec![0.0; prices.len()];

    unsafe {
        let ret_code = TA_RSI(
            0,
            (prices.len() - 1) as i32,
            prices.as_ptr(),
            period as i32,
            &mut out_begin,
            &mut out_size,
            output.as_mut_ptr(),
        );

        if ret_code != 0 {
            return Err(format!("TA_RSI returned error code: {}", ret_code));
        }
    }

    output.truncate(out_size as usize);
    Ok(output)
}

/// Safe wrapper for MACD
pub fn calculate_macd(
    prices: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>), String> {
    if prices.len() < slow_period {
        return Err("Insufficient data for MACD calculation".to_string());
    }

    let mut out_begin = 0i32;
    let mut out_size = 0i32;
    let mut macd = vec![0.0; prices.len()];
    let mut signal = vec![0.0; prices.len()];
    let mut hist = vec![0.0; prices.len()];

    unsafe {
        let ret_code = TA_MACD(
            0,
            (prices.len() - 1) as i32,
            prices.as_ptr(),
            fast_period as i32,
            slow_period as i32,
            signal_period as i32,
            &mut out_begin,
            &mut out_size,
            macd.as_mut_ptr(),
            signal.as_mut_ptr(),
            hist.as_mut_ptr(),
        );

        if ret_code != 0 {
            return Err(format!("TA_MACD returned error code: {}", ret_code));
        }
    }

    macd.truncate(out_size as usize);
    signal.truncate(out_size as usize);
    hist.truncate(out_size as usize);

    Ok((macd, signal, hist))
}

fn main() {
    // Simulating Bitcoin prices for 30 days
    let prices: Vec<f64> = (0..30)
        .map(|i| 42000.0 + (i as f64 * 50.0) + ((i * 7) % 13) as f64 * 100.0)
        .collect();

    println!("=== Technical Analysis BTC/USD ===\n");

    // Calculate RSI
    match calculate_rsi(&prices, 14) {
        Ok(rsi_values) => {
            if let Some(current_rsi) = rsi_values.last() {
                println!("RSI(14): {:.2}", current_rsi);

                if *current_rsi > 70.0 {
                    println!("  📊 Status: Overbought");
                } else if *current_rsi < 30.0 {
                    println!("  📊 Status: Oversold");
                } else {
                    println!("  📊 Status: Neutral zone");
                }
            }
        }
        Err(e) => eprintln!("RSI calculation error: {}", e),
    }

    println!();

    // Calculate MACD
    match calculate_macd(&prices, 12, 26, 9) {
        Ok((macd, signal, histogram)) => {
            if let (Some(&m), Some(&s), Some(&h)) = (macd.last(), signal.last(), histogram.last()) {
                println!("MACD(12,26,9):");
                println!("  MACD: {:.2}", m);
                println!("  Signal: {:.2}", s);
                println!("  Histogram: {:.2}", h);

                if h > 0.0 {
                    println!("  📈 Bullish signal");
                } else {
                    println!("  📉 Bearish signal");
                }
            }
        }
        Err(e) => eprintln!("MACD calculation error: {}", e),
    }
}
```

## Working with Strings via FFI

Strings are one of the complex areas of FFI, as Rust and C have different string representations.

### Passing Strings from Rust to C

```rust
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

extern "C" {
    // C function accepting a string
    fn log_trade(symbol: *const c_char, price: f64, quantity: f64);
}

fn safe_log_trade(symbol: &str, price: f64, quantity: f64) {
    // Convert Rust string to C string
    let c_symbol = CString::new(symbol).expect("CString creation error");

    unsafe {
        log_trade(c_symbol.as_ptr(), price, quantity);
    }
}

fn main() {
    safe_log_trade("BTC/USD", 42500.0, 0.5);
    safe_log_trade("ETH/USD", 2200.0, 2.0);
}
```

### Getting Strings from C to Rust

```rust
use std::ffi::CStr;
use std::os::raw::c_char;

extern "C" {
    // C function returning a string
    fn get_exchange_name() -> *const c_char;
}

fn safe_get_exchange_name() -> String {
    unsafe {
        let c_str = CStr::from_ptr(get_exchange_name());
        c_str.to_string_lossy().into_owned()
    }
}

fn main() {
    let exchange = safe_get_exchange_name();
    println!("Connected to exchange: {}", exchange);
}
```

## Memory Management with FFI

It's critically important to understand who owns the memory when working with FFI.

### Memory Management Rules

```rust
use std::ffi::CString;
use std::os::raw::c_char;
use std::mem;

extern "C" {
    // C function allocates memory, Rust must free it
    fn create_order_id() -> *mut c_char;

    // C function frees memory allocated in C
    fn free_order_id(ptr: *mut c_char);
}

/// Safe wrapper with automatic memory management
struct OrderId {
    ptr: *mut c_char,
}

impl OrderId {
    fn new() -> Self {
        unsafe {
            OrderId {
                ptr: create_order_id(),
            }
        }
    }

    fn as_str(&self) -> &str {
        unsafe {
            std::ffi::CStr::from_ptr(self.ptr)
                .to_str()
                .unwrap_or("invalid")
        }
    }
}

impl Drop for OrderId {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                free_order_id(self.ptr);
            }
        }
    }
}

fn main() {
    let order_id = OrderId::new();
    println!("Created order with ID: {}", order_id.as_str());

    // Memory is automatically freed when going out of scope
}
```

## Error Handling via FFI

C typically returns error codes, while Rust uses `Result`. Here's the conversion pattern:

```rust
use std::os::raw::c_int;

extern "C" {
    fn execute_trade(symbol: *const i8, quantity: f64) -> c_int;
}

#[derive(Debug)]
enum TradeError {
    InvalidSymbol,
    InsufficientFunds,
    ExchangeError,
    Unknown(i32),
}

impl TradeError {
    fn from_code(code: i32) -> Self {
        match code {
            -1 => TradeError::InvalidSymbol,
            -2 => TradeError::InsufficientFunds,
            -3 => TradeError::ExchangeError,
            other => TradeError::Unknown(other),
        }
    }
}

fn safe_execute_trade(symbol: &str, quantity: f64) -> Result<(), TradeError> {
    let c_symbol = std::ffi::CString::new(symbol)
        .map_err(|_| TradeError::InvalidSymbol)?;

    unsafe {
        let result = execute_trade(c_symbol.as_ptr(), quantity);

        if result == 0 {
            Ok(())
        } else {
            Err(TradeError::from_code(result))
        }
    }
}

fn main() {
    match safe_execute_trade("BTC/USD", 0.5) {
        Ok(()) => println!("✅ Trade executed successfully"),
        Err(e) => eprintln!("❌ Trade execution error: {:?}", e),
    }
}
```

## Creating a C Library in Rust (Reverse FFI)

Rust code can also be compiled as a C library for use in other languages.

### Rust Library for Indicator Calculation

```rust
use std::os::raw::c_double;
use std::slice;

/// Exported function for EMA calculation
#[no_mangle]
pub extern "C" fn calculate_ema(
    prices: *const c_double,
    length: usize,
    period: usize,
) -> c_double {
    if prices.is_null() || length < period || period == 0 {
        return 0.0;
    }

    let prices_slice = unsafe {
        slice::from_raw_parts(prices, length)
    };

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = prices_slice[0];

    for &price in &prices_slice[1..] {
        ema = (price * multiplier) + (ema * (1.0 - multiplier));
    }

    ema
}

/// Free array allocated in Rust
#[no_mangle]
pub extern "C" fn free_array(ptr: *mut c_double, length: usize) {
    if !ptr.is_null() {
        unsafe {
            Vec::from_raw_parts(ptr, length, length);
            // Vector is automatically freed here
        }
    }
}
```

### Header File for C (indicators.h)

```c
// indicators.h
#ifndef INDICATORS_H
#define INDICATORS_H

#include <stddef.h>

// Calculate Exponential Moving Average
double calculate_ema(const double* prices, size_t length, size_t period);

// Free array
void free_array(double* ptr, size_t length);

#endif // INDICATORS_H
```

### Building Rust Library for C

In `Cargo.toml`:

```toml
[lib]
name = "indicators"
crate-type = ["cdylib", "staticlib"]
```

Compilation:

```bash
cargo build --release
```

Usage from C:

```c
#include <stdio.h>
#include "indicators.h"

int main() {
    double prices[] = {100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0};
    size_t length = sizeof(prices) / sizeof(prices[0]);

    double ema = calculate_ema(prices, length, 5);

    printf("EMA(5) = %.2f\n", ema);

    return 0;
}
```

## Thread Safety and FFI

When working with FFI in a multithreaded environment, it's important to consider the thread safety of C libraries.

```rust
use std::sync::Mutex;
use std::os::raw::c_int;

extern "C" {
    // NOT thread-safe C function
    fn legacy_calculate_price(amount: f64) -> f64;
}

// Use Mutex for safe access
lazy_static::lazy_static! {
    static ref PRICE_CALC_LOCK: Mutex<()> = Mutex::new(());
}

fn safe_calculate_price(amount: f64) -> f64 {
    let _guard = PRICE_CALC_LOCK.lock().unwrap();

    unsafe {
        legacy_calculate_price(amount)
    }
}

fn main() {
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let price = safe_calculate_price(100.0 * i as f64);
                println!("Thread {}: Price = {:.2}", i, price);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **FFI** | Foreign Function Interface — interface for calling functions from other languages |
| **extern "C"** | Block for declaring external C functions |
| **unsafe** | All FFI calls require an unsafe block |
| **#[repr(C)]** | Attribute for C-compatible structure layout |
| **CString/CStr** | Types for working with C strings |
| **#[no_mangle]** | Attribute for exporting Rust functions to C |
| **Ownership** | Critically important to understand memory ownership |
| **Error handling** | Converting C error codes to Rust Result |

## Practical Exercises

1. **Safe Wrapper**: Create a safe Rust wrapper for the following C function:
   ```c
   // Calculates Bollinger Bands
   int calculate_bollinger_bands(
       const double* prices,
       int length,
       int period,
       double std_dev,
       double* upper_band,
       double* middle_band,
       double* lower_band
   );
   ```
   Handle all possible errors and create an idiomatic Rust API.

2. **Rust Library for C**: Write a function in Rust to calculate ATR (Average True Range) and export it as a C library. Create a header file and usage example in C.

3. **Legacy System Integration**: Imagine you have an old trading system in C with functions:
   ```c
   void* create_order(const char* symbol, double price, double quantity);
   int get_order_status(void* order);
   void cancel_order(void* order);
   void free_order(void* order);
   ```
   Create a safe Rust wrapper with automatic memory management via RAII (Drop trait).

4. **Callback Functions**: Implement a callback mechanism for receiving price updates:
   ```rust
   // C side calls Rust callback on price update
   extern "C" fn price_update_callback(symbol: *const c_char, price: f64);
   ```
   Integrate this with Rust closures for handling updates.

## Homework

1. **FFI Calculator Library**: Create a simple C library with functions for calculating basic indicators (SMA, EMA, RSI) and integrate it into a Rust application with full error handling.

2. **Trading Bridge**: Write a "bridge" between a Rust trading strategy and a legacy C exchange API. Implement:
   - Connection to exchange via C API
   - Receiving market data
   - Placing orders
   - Error handling and reconnection
   - Logging all operations

3. **Performance Benchmark**: Compare performance of:
   - Native Rust SMA implementation
   - FFI call to C library for SMA
   - TA-Lib via FFI

   Measure time on different data sizes (100, 1000, 10000, 100000 points).

4. **Safe Wrapper Pattern**: Create a generalized wrapper library for safe FFI work in the trading context, including:
   - Automatic memory management
   - Converting C errors to Rust Result
   - Thread-safe wrappers
   - Documentation and usage examples

## Navigation

[← Previous day](../294-overfitting-strategy-optimization/en.md) | [Next day →](../320-valgrind-and-heaptrack/en.md)
