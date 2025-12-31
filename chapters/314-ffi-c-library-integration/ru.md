# День 314: FFI: интеграция с C библиотеками

## Аналогия из трейдинга

Представь, что ты разрабатываешь торговую платформу на Rust, но существует мощная библиотека для технического анализа, написанная на C — например, TA-Lib (Technical Analysis Library). В ней есть сотни индикаторов, оптимизированных и протестированных годами.

Вместо того чтобы переписывать всё с нуля на Rust, ты можешь использовать **FFI (Foreign Function Interface)** — это как иметь переводчика между двумя языками:

- **Rust** — твой основной язык, безопасный и быстрый
- **C библиотека** — проверенные временем алгоритмы
- **FFI** — мост между ними

Это аналогично тому, как крупные банки интегрируют старые торговые системы (написанные на C/C++) с новыми сервисами. Вместо полной переписки они создают интерфейсы для взаимодействия.

## Что такое FFI?

**FFI (Foreign Function Interface)** — это механизм, позволяющий Rust вызывать функции, написанные на других языках (преимущественно C), и наоборот.

### Почему это важно в трейдинге?

| Сценарий | Причина использования FFI |
|----------|---------------------------|
| **Легаси код** | Существующие торговые системы на C/C++ |
| **Оптимизированные библиотеки** | TA-Lib, QuickFIX (FIX protocol), libta |
| **Платформенные API** | Системные вызовы, драйверы биржевых шлюзов |
| **Производительность** | Критичные секции на низкоуровневом C |
| **Интеграция** | Подключение к брокерским API на C |

## Основы FFI: вызов C функций из Rust

### Простой пример: вызов функции из libc

```rust
use std::ffi::CString;
use std::os::raw::c_char;

// Объявляем внешнюю функцию из libc
extern "C" {
    fn strlen(s: *const c_char) -> usize;
}

fn main() {
    // Создаём C-совместимую строку
    let ticker = CString::new("BTCUSD").expect("CString creation failed");

    // Вызов C функции (unsafe!)
    unsafe {
        let len = strlen(ticker.as_ptr());
        println!("Длина тикера '{}': {}", ticker.to_str().unwrap(), len);
    }
}
```

### Почему `unsafe`?

Rust не может гарантировать безопасность кода на C:
- **Нет проверок границ массивов**
- **Возможны null указатели**
- **Нет контроля времени жизни**
- **Возможна порча памяти**

Поэтому все вызовы FFI требуют `unsafe` блока.

## Пример: интеграция с простой C библиотекой для расчёта SMA

Представим, что у нас есть C библиотека для расчёта Simple Moving Average:

### C код (sma.h и sma.c)

```c
// sma.h
#ifndef SMA_H
#define SMA_H

#include <stddef.h>

// Расчёт Simple Moving Average
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

### Rust интеграция

```rust
use std::os::raw::c_double;

// Объявляем внешнюю C функцию
extern "C" {
    fn calculate_sma(prices: *const c_double, length: usize, period: usize) -> c_double;
}

/// Безопасная обёртка над C функцией
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
        println!("SMA(5) для BTC: ${:.2}", sma_5);
    }

    if let Some(sma_10) = sma(&btc_prices, 10) {
        println!("SMA(10) для BTC: ${:.2}", sma_10);
    }
}
```

## Типы данных FFI: маппинг между Rust и C

| Rust | C | Описание |
|------|---|----------|
| `i8` | `int8_t` | 8-битное знаковое целое |
| `u8` | `uint8_t` | 8-битное беззнаковое целое |
| `i32` | `int32_t` | 32-битное знаковое целое |
| `f64` | `double` | Двойной точности float |
| `*const T` | `const T*` | Константный указатель |
| `*mut T` | `T*` | Изменяемый указатель |
| `()` | `void` | Отсутствие значения |

### Пример: структуры данных

```rust
use std::os::raw::{c_char, c_double, c_int};

// Rust структура, совместимая с C
#[repr(C)]
pub struct Candle {
    timestamp: i64,
    open: c_double,
    high: c_double,
    low: c_double,
    close: c_double,
    volume: c_double,
}

// C функция для обработки свечи
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
        1 => println!("📈 Сигнал на покупку"),
        -1 => println!("📉 Сигнал на продажу"),
        _ => println!("⏸️  Удержание позиции"),
    }
}
```

## Продвинутый пример: интеграция с TA-Lib

TA-Lib — это популярная библиотека технического анализа на C. Вот как её можно использовать в Rust:

### Объявление функций TA-Lib

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

/// Безопасная обёртка для RSI
pub fn calculate_rsi(prices: &[f64], period: usize) -> Result<Vec<f64>, String> {
    if prices.len() < period {
        return Err("Недостаточно данных для расчёта RSI".to_string());
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
            return Err(format!("TA_RSI вернула код ошибки: {}", ret_code));
        }
    }

    output.truncate(out_size as usize);
    Ok(output)
}

/// Безопасная обёртка для MACD
pub fn calculate_macd(
    prices: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>), String> {
    if prices.len() < slow_period {
        return Err("Недостаточно данных для расчёта MACD".to_string());
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
            return Err(format!("TA_MACD вернула код ошибки: {}", ret_code));
        }
    }

    macd.truncate(out_size as usize);
    signal.truncate(out_size as usize);
    hist.truncate(out_size as usize);

    Ok((macd, signal, hist))
}

fn main() {
    // Имитация цен Bitcoin за 30 дней
    let prices: Vec<f64> = (0..30)
        .map(|i| 42000.0 + (i as f64 * 50.0) + ((i * 7) % 13) as f64 * 100.0)
        .collect();

    println!("=== Технический анализ BTC/USD ===\n");

    // Расчёт RSI
    match calculate_rsi(&prices, 14) {
        Ok(rsi_values) => {
            if let Some(current_rsi) = rsi_values.last() {
                println!("RSI(14): {:.2}", current_rsi);

                if *current_rsi > 70.0 {
                    println!("  📊 Статус: Перекупленность (overbought)");
                } else if *current_rsi < 30.0 {
                    println!("  📊 Статус: Перепроданность (oversold)");
                } else {
                    println!("  📊 Статус: Нейтральная зона");
                }
            }
        }
        Err(e) => eprintln!("Ошибка расчёта RSI: {}", e),
    }

    println!();

    // Расчёт MACD
    match calculate_macd(&prices, 12, 26, 9) {
        Ok((macd, signal, histogram)) => {
            if let (Some(&m), Some(&s), Some(&h)) = (macd.last(), signal.last(), histogram.last()) {
                println!("MACD(12,26,9):");
                println!("  MACD: {:.2}", m);
                println!("  Signal: {:.2}", s);
                println!("  Histogram: {:.2}", h);

                if h > 0.0 {
                    println!("  📈 Бычий сигнал (bullish)");
                } else {
                    println!("  📉 Медвежий сигнал (bearish)");
                }
            }
        }
        Err(e) => eprintln!("Ошибка расчёта MACD: {}", e),
    }
}
```

## Работа со строками через FFI

Строки — одна из сложных областей FFI, так как Rust и C имеют разные представления строк.

### Передача строк из Rust в C

```rust
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

extern "C" {
    // C функция, принимающая строку
    fn log_trade(symbol: *const c_char, price: f64, quantity: f64);
}

fn safe_log_trade(symbol: &str, price: f64, quantity: f64) {
    // Преобразуем Rust строку в C строку
    let c_symbol = CString::new(symbol).expect("Ошибка создания CString");

    unsafe {
        log_trade(c_symbol.as_ptr(), price, quantity);
    }
}

fn main() {
    safe_log_trade("BTC/USD", 42500.0, 0.5);
    safe_log_trade("ETH/USD", 2200.0, 2.0);
}
```

### Получение строк из C в Rust

```rust
use std::ffi::CStr;
use std::os::raw::c_char;

extern "C" {
    // C функция, возвращающая строку
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
    println!("Подключено к бирже: {}", exchange);
}
```

## Управление памятью при FFI

Критически важно понимать, кто владеет памятью при работе с FFI.

### Правила управления памятью

```rust
use std::ffi::CString;
use std::os::raw::c_char;
use std::mem;

extern "C" {
    // C функция аллоцирует память, Rust должен её освободить
    fn create_order_id() -> *mut c_char;

    // C функция освобождает память, выделенную в C
    fn free_order_id(ptr: *mut c_char);
}

/// Безопасная обёртка с автоматическим управлением памятью
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
    println!("Создан ордер с ID: {}", order_id.as_str());

    // Память автоматически освобождается при выходе из области видимости
}
```

## Обработка ошибок через FFI

C обычно возвращает коды ошибок, а Rust использует `Result`. Вот паттерн преобразования:

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
        Ok(()) => println!("✅ Сделка выполнена успешно"),
        Err(e) => eprintln!("❌ Ошибка выполнения сделки: {:?}", e),
    }
}
```

## Создание C библиотеки на Rust (обратная FFI)

Rust код также может быть скомпилирован как C библиотека для использования в других языках.

### Rust библиотека для расчёта индикаторов

```rust
use std::os::raw::c_double;
use std::slice;

/// Экспортируемая функция для расчёта EMA
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

/// Освобождение массива, аллоцированного в Rust
#[no_mangle]
pub extern "C" fn free_array(ptr: *mut c_double, length: usize) {
    if !ptr.is_null() {
        unsafe {
            Vec::from_raw_parts(ptr, length, length);
            // Вектор автоматически освобождается здесь
        }
    }
}
```

### Файл заголовка для C (indicators.h)

```c
// indicators.h
#ifndef INDICATORS_H
#define INDICATORS_H

#include <stddef.h>

// Расчёт Exponential Moving Average
double calculate_ema(const double* prices, size_t length, size_t period);

// Освобождение массива
void free_array(double* ptr, size_t length);

#endif // INDICATORS_H
```

### Сборка Rust библиотеки для C

В `Cargo.toml`:

```toml
[lib]
name = "indicators"
crate-type = ["cdylib", "staticlib"]
```

Компиляция:

```bash
cargo build --release
```

Использование из C:

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

## Потокобезопасность и FFI

При работе с FFI в многопоточной среде важно учитывать потокобезопасность C библиотек.

```rust
use std::sync::Mutex;
use std::os::raw::c_int;

extern "C" {
    // НЕ потокобезопасная C функция
    fn legacy_calculate_price(amount: f64) -> f64;
}

// Используем Mutex для безопасного доступа
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

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **FFI** | Foreign Function Interface — интерфейс для вызова функций из других языков |
| **extern "C"** | Блок объявления внешних C функций |
| **unsafe** | Все FFI вызовы требуют unsafe блока |
| **#[repr(C)]** | Атрибут для совместимости структур с C |
| **CString/CStr** | Типы для работы со строками C |
| **#[no_mangle]** | Атрибут для экспорта Rust функций в C |
| **Ownership** | Критически важно понимать владение памятью |
| **Error handling** | Преобразование C кодов ошибок в Rust Result |

## Практические задания

1. **Безопасная обёртка**: Создай безопасную Rust обёртку для следующей C функции:
   ```c
   // Вычисляет Bollinger Bands
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
   Обработай все возможные ошибки и создай идиоматичный Rust API.

2. **Rust библиотека для C**: Напиши на Rust функцию расчёта ATR (Average True Range) и экспортируй её как C библиотеку. Создай заголовочный файл и пример использования на C.

3. **Интеграция с legacy системой**: Представь, что у тебя есть старая торговая система на C с функциями:
   ```c
   void* create_order(const char* symbol, double price, double quantity);
   int get_order_status(void* order);
   void cancel_order(void* order);
   void free_order(void* order);
   ```
   Создай безопасную Rust обёртку с автоматическим управлением памятью через RAII (Drop trait).

4. **Callback функции**: Реализуй механизм callback'ов для получения обновлений цен:
   ```rust
   // C сторона вызывает Rust callback при обновлении цены
   extern "C" fn price_update_callback(symbol: *const c_char, price: f64);
   ```
   Интегрируй это с Rust closure для обработки обновлений.

## Домашнее задание

1. **FFI Calculator Library**: Создай простую C библиотеку с функциями для расчёта основных индикаторов (SMA, EMA, RSI) и интегрируй её в Rust приложение с полной обработкой ошибок.

2. **Trading Bridge**: Напиши "мост" между Rust торговой стратегией и legacy C API биржи. Реализуй:
   - Подключение к бирже через C API
   - Получение рыночных данных
   - Размещение ордеров
   - Обработку ошибок и переподключения
   - Логирование всех операций

3. **Performance Benchmark**: Сравни производительность:
   - Нативной Rust реализации SMA
   - FFI вызова C библиотеки для SMA
   - TA-Lib через FFI

   Измерь время на разных размерах данных (100, 1000, 10000, 100000 точек).

4. **Safe Wrapper Pattern**: Создай обобщённую библиотеку-обёртку для безопасной работы с FFI в контексте трейдинга, включающую:
   - Автоматическое управление памятью
   - Преобразование ошибок C в Rust Result
   - Потокобезопасные обёртки
   - Документацию и примеры использования

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md) | [Следующий день →](../320-valgrind-and-heaptrack/ru.md)
