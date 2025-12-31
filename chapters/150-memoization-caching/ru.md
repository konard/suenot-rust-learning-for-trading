# День 150: Мемоизация: кешируем результаты

## Аналогия из трейдинга

Представь, что ты каждый день рассчитываешь скользящую среднюю (SMA) для 200 дней. Каждый раз пересчитывать все 200 значений — это расточительно. Умный трейдер **запоминает** вчерашний результат и просто добавляет новую цену, убирая самую старую. Это и есть **мемоизация** — сохранение результатов вычислений для повторного использования.

В торговле это особенно важно:
- Расчёт индикаторов (RSI, MACD, Bollinger Bands) требует много вычислений
- Оценка риска портфеля может занимать секунды
- Бэктестинг стратегий повторяет одни и те же расчёты миллионы раз

## Что такое мемоизация?

**Мемоизация** — это техника оптимизации, при которой результаты функции сохраняются в кеше. При повторном вызове с теми же аргументами функция возвращает сохранённый результат вместо повторного вычисления.

```rust
use std::collections::HashMap;

fn main() {
    // Без мемоизации: каждый вызов пересчитывает
    let fib_10 = fibonacci_naive(10);
    println!("Fibonacci(10) = {}", fib_10);

    // С мемоизацией: результаты кешируются
    let mut cache: HashMap<u64, u64> = HashMap::new();
    let fib_10_cached = fibonacci_memo(10, &mut cache);
    println!("Fibonacci(10) cached = {}", fib_10_cached);
}

// Наивная реализация — экспоненциальная сложность O(2^n)
fn fibonacci_naive(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    fibonacci_naive(n - 1) + fibonacci_naive(n - 2)
}

// С мемоизацией — линейная сложность O(n)
fn fibonacci_memo(n: u64, cache: &mut HashMap<u64, u64>) -> u64 {
    if n <= 1 {
        return n;
    }

    if let Some(&result) = cache.get(&n) {
        return result;  // Возвращаем кешированный результат
    }

    let result = fibonacci_memo(n - 1, cache) + fibonacci_memo(n - 2, cache);
    cache.insert(n, result);  // Сохраняем в кеш
    result
}
```

## Мемоизация для торговых индикаторов

### Кеширование SMA (Simple Moving Average)

```rust
use std::collections::HashMap;

struct SmaCalculator {
    cache: HashMap<String, f64>,  // Ключ: "symbol:period:timestamp"
}

impl SmaCalculator {
    fn new() -> Self {
        SmaCalculator {
            cache: HashMap::new(),
        }
    }

    fn calculate(&mut self, symbol: &str, period: usize, prices: &[f64], timestamp: u64) -> f64 {
        let key = format!("{}:{}:{}", symbol, period, timestamp);

        // Проверяем кеш
        if let Some(&cached) = self.cache.get(&key) {
            println!("  [CACHE HIT] SMA для {} уже рассчитан", symbol);
            return cached;
        }

        println!("  [CALCULATING] Рассчитываем SMA для {}", symbol);

        // Вычисляем SMA
        if prices.len() < period {
            return 0.0;
        }

        let slice = &prices[prices.len() - period..];
        let sum: f64 = slice.iter().sum();
        let sma = sum / period as f64;

        // Сохраняем в кеш
        self.cache.insert(key, sma);
        sma
    }

    fn cache_size(&self) -> usize {
        self.cache.len()
    }

    fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

fn main() {
    let mut calculator = SmaCalculator::new();

    let btc_prices = vec![42000.0, 42100.0, 42050.0, 42200.0, 42150.0];

    // Первый вызов — вычисляет
    let sma1 = calculator.calculate("BTC", 3, &btc_prices, 1000);
    println!("SMA(3) = {:.2}\n", sma1);

    // Повторный вызов — из кеша
    let sma2 = calculator.calculate("BTC", 3, &btc_prices, 1000);
    println!("SMA(3) = {:.2}\n", sma2);

    // Другой период — новый расчёт
    let sma3 = calculator.calculate("BTC", 5, &btc_prices, 1000);
    println!("SMA(5) = {:.2}\n", sma3);

    println!("Размер кеша: {} записей", calculator.cache_size());
}
```

### Кеширование RSI (Relative Strength Index)

```rust
use std::collections::HashMap;

struct RsiCache {
    cache: HashMap<String, f64>,
    calculations_saved: u32,
}

impl RsiCache {
    fn new() -> Self {
        RsiCache {
            cache: HashMap::new(),
            calculations_saved: 0,
        }
    }

    fn get_or_calculate(&mut self, key: &str, prices: &[f64], period: usize) -> f64 {
        if let Some(&cached) = self.cache.get(key) {
            self.calculations_saved += 1;
            return cached;
        }

        let rsi = self.calculate_rsi(prices, period);
        self.cache.insert(key.to_string(), rsi);
        rsi
    }

    fn calculate_rsi(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() < period + 1 {
            return 50.0;  // Нейтральное значение при недостатке данных
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..=period {
            let change = prices[prices.len() - period - 1 + i] - prices[prices.len() - period - 2 + i];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(change.abs());
            }
        }

        let avg_gain: f64 = gains.iter().sum::<f64>() / period as f64;
        let avg_loss: f64 = losses.iter().sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    fn stats(&self) -> (usize, u32) {
        (self.cache.len(), self.calculations_saved)
    }
}

fn main() {
    let mut rsi_cache = RsiCache::new();

    let prices = vec![
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
    ];

    // Симулируем множественные запросы
    for _ in 0..100 {
        let _rsi = rsi_cache.get_or_calculate("BTC:14:1000", &prices, 14);
    }

    let (cache_size, saved) = rsi_cache.stats();
    println!("Размер кеша: {}", cache_size);
    println!("Сэкономлено вычислений: {}", saved);
}
```

## Структура для мемоизации с TTL (Time To Live)

В торговле данные быстро устаревают. Нужен кеш с временем жизни:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    ttl: Duration,
}

impl<T: Clone> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        CacheEntry {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    fn get(&self) -> Option<T> {
        if self.is_expired() {
            None
        } else {
            Some(self.value.clone())
        }
    }
}

struct TradingCache {
    indicators: HashMap<String, CacheEntry<f64>>,
    default_ttl: Duration,
}

impl TradingCache {
    fn new(ttl_seconds: u64) -> Self {
        TradingCache {
            indicators: HashMap::new(),
            default_ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn get(&self, key: &str) -> Option<f64> {
        self.indicators.get(key).and_then(|entry| entry.get())
    }

    fn set(&mut self, key: String, value: f64) {
        let entry = CacheEntry::new(value, self.default_ttl);
        self.indicators.insert(key, entry);
    }

    fn get_or_compute<F>(&mut self, key: &str, compute: F) -> f64
    where
        F: FnOnce() -> f64,
    {
        if let Some(value) = self.get(key) {
            return value;
        }

        let value = compute();
        self.set(key.to_string(), value);
        value
    }

    fn cleanup_expired(&mut self) {
        self.indicators.retain(|_, entry| !entry.is_expired());
    }
}

fn main() {
    let mut cache = TradingCache::new(60);  // TTL = 60 секунд

    // Кешируем расчёт SMA
    let sma = cache.get_or_compute("BTC:SMA:20", || {
        println!("Вычисляем SMA...");
        42150.0  // Результат расчёта
    });
    println!("SMA = {}", sma);

    // Повторный запрос — из кеша
    let sma_cached = cache.get_or_compute("BTC:SMA:20", || {
        println!("Это не должно выполниться!");
        0.0
    });
    println!("SMA (cached) = {}", sma_cached);
}
```

## Мемоизация для расчёта риска портфеля

```rust
use std::collections::HashMap;

struct PortfolioRiskCalculator {
    var_cache: HashMap<String, f64>,  // Value at Risk
    correlation_cache: HashMap<String, f64>,
    calculation_count: u32,
    cache_hit_count: u32,
}

impl PortfolioRiskCalculator {
    fn new() -> Self {
        PortfolioRiskCalculator {
            var_cache: HashMap::new(),
            correlation_cache: HashMap::new(),
            calculation_count: 0,
            cache_hit_count: 0,
        }
    }

    fn calculate_var(&mut self, portfolio_key: &str, returns: &[f64], confidence: f64) -> f64 {
        let cache_key = format!("{}:{:.2}", portfolio_key, confidence);

        if let Some(&cached_var) = self.var_cache.get(&cache_key) {
            self.cache_hit_count += 1;
            return cached_var;
        }

        self.calculation_count += 1;

        // Упрощённый расчёт VaR (историческая симуляция)
        let mut sorted_returns = returns.to_vec();
        sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((1.0 - confidence) * sorted_returns.len() as f64) as usize;
        let var = sorted_returns.get(index).copied().unwrap_or(0.0).abs();

        self.var_cache.insert(cache_key, var);
        var
    }

    fn calculate_correlation(&mut self, asset_a: &str, asset_b: &str, returns_a: &[f64], returns_b: &[f64]) -> f64 {
        // Создаём симметричный ключ
        let cache_key = if asset_a < asset_b {
            format!("{}:{}", asset_a, asset_b)
        } else {
            format!("{}:{}", asset_b, asset_a)
        };

        if let Some(&cached) = self.correlation_cache.get(&cache_key) {
            self.cache_hit_count += 1;
            return cached;
        }

        self.calculation_count += 1;

        // Расчёт корреляции Пирсона
        let n = returns_a.len().min(returns_b.len());
        if n == 0 {
            return 0.0;
        }

        let mean_a: f64 = returns_a.iter().take(n).sum::<f64>() / n as f64;
        let mean_b: f64 = returns_b.iter().take(n).sum::<f64>() / n as f64;

        let mut cov = 0.0;
        let mut var_a = 0.0;
        let mut var_b = 0.0;

        for i in 0..n {
            let diff_a = returns_a[i] - mean_a;
            let diff_b = returns_b[i] - mean_b;
            cov += diff_a * diff_b;
            var_a += diff_a * diff_a;
            var_b += diff_b * diff_b;
        }

        let correlation = if var_a > 0.0 && var_b > 0.0 {
            cov / (var_a.sqrt() * var_b.sqrt())
        } else {
            0.0
        };

        self.correlation_cache.insert(cache_key, correlation);
        correlation
    }

    fn stats(&self) -> (u32, u32, f64) {
        let total = self.calculation_count + self.cache_hit_count;
        let hit_rate = if total > 0 {
            (self.cache_hit_count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        (self.calculation_count, self.cache_hit_count, hit_rate)
    }
}

fn main() {
    let mut calculator = PortfolioRiskCalculator::new();

    // Симулируем доходности
    let btc_returns = vec![-0.02, 0.03, -0.01, 0.02, 0.01, -0.015, 0.025];
    let eth_returns = vec![-0.03, 0.04, -0.02, 0.03, 0.015, -0.02, 0.035];

    // Множественные расчёты VaR
    for _ in 0..10 {
        let var_95 = calculator.calculate_var("portfolio_1", &btc_returns, 0.95);
        let var_99 = calculator.calculate_var("portfolio_1", &btc_returns, 0.99);
        println!("VaR 95%: {:.4}, VaR 99%: {:.4}", var_95, var_99);
    }

    // Корреляции
    let corr = calculator.calculate_correlation("BTC", "ETH", &btc_returns, &eth_returns);
    let corr_reverse = calculator.calculate_correlation("ETH", "BTC", &eth_returns, &btc_returns);
    println!("\nКорреляция BTC-ETH: {:.4}", corr);
    println!("Корреляция ETH-BTC (из кеша): {:.4}", corr_reverse);

    let (calcs, hits, rate) = calculator.stats();
    println!("\nСтатистика:");
    println!("  Вычислений: {}", calcs);
    println!("  Попаданий в кеш: {}", hits);
    println!("  Процент попаданий: {:.1}%", rate);
}
```

## Паттерн Lazy Evaluation с мемоизацией

```rust
use std::cell::RefCell;

struct LazyIndicator<T, F>
where
    F: Fn() -> T,
{
    compute: F,
    cached: RefCell<Option<T>>,
}

impl<T: Clone, F: Fn() -> T> LazyIndicator<T, F> {
    fn new(compute: F) -> Self {
        LazyIndicator {
            compute,
            cached: RefCell::new(None),
        }
    }

    fn get(&self) -> T {
        let mut cached = self.cached.borrow_mut();
        if cached.is_none() {
            *cached = Some((self.compute)());
        }
        cached.as_ref().unwrap().clone()
    }

    fn invalidate(&self) {
        *self.cached.borrow_mut() = None;
    }
}

fn main() {
    let prices = vec![42000.0, 42100.0, 42050.0, 42200.0, 42150.0];

    // Создаём ленивый индикатор
    let lazy_sma = LazyIndicator::new(|| {
        println!("Вычисляем SMA...");
        let sum: f64 = prices.iter().sum();
        sum / prices.len() as f64
    });

    println!("Первый вызов:");
    let sma1 = lazy_sma.get();
    println!("SMA = {:.2}\n", sma1);

    println!("Второй вызов (из кеша):");
    let sma2 = lazy_sma.get();
    println!("SMA = {:.2}\n", sma2);

    println!("После инвалидации:");
    lazy_sma.invalidate();
    let sma3 = lazy_sma.get();
    println!("SMA = {:.2}", sma3);
}
```

## LRU кеш для торговых данных

```rust
use std::collections::{HashMap, VecDeque};

struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, V>,
    order: VecDeque<K>,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> LruCache<K, V> {
    fn new(capacity: usize) -> Self {
        LruCache {
            capacity,
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn get(&mut self, key: &K) -> Option<V> {
        if self.map.contains_key(key) {
            // Перемещаем в конец (самый свежий)
            self.order.retain(|k| k != key);
            self.order.push_back(key.clone());
            self.map.get(key).cloned()
        } else {
            None
        }
    }

    fn put(&mut self, key: K, value: V) {
        if self.map.contains_key(&key) {
            self.order.retain(|k| k != &key);
        } else if self.map.len() >= self.capacity {
            // Удаляем самый старый
            if let Some(oldest) = self.order.pop_front() {
                self.map.remove(&oldest);
            }
        }

        self.map.insert(key.clone(), value);
        self.order.push_back(key);
    }

    fn len(&self) -> usize {
        self.map.len()
    }
}

fn main() {
    let mut cache: LruCache<String, f64> = LruCache::new(3);  // Максимум 3 элемента

    cache.put("BTC".to_string(), 42000.0);
    cache.put("ETH".to_string(), 2500.0);
    cache.put("SOL".to_string(), 100.0);

    println!("Кеш после добавления 3 элементов:");
    println!("  BTC: {:?}", cache.get(&"BTC".to_string()));
    println!("  Размер: {}", cache.len());

    // Добавляем 4-й элемент — вытесняет самый старый (ETH, т.к. BTC был запрошен)
    cache.put("XRP".to_string(), 0.5);

    println!("\nПосле добавления XRP:");
    println!("  ETH: {:?}", cache.get(&"ETH".to_string()));  // None — вытеснен
    println!("  BTC: {:?}", cache.get(&"BTC".to_string()));  // Есть
    println!("  XRP: {:?}", cache.get(&"XRP".to_string()));  // Есть
}
```

## Практический пример: система торговых сигналов с кешированием

```rust
use std::collections::HashMap;

struct TradingSignalSystem {
    indicator_cache: HashMap<String, f64>,
    signal_cache: HashMap<String, (String, f64)>,  // (сигнал, уверенность)
}

impl TradingSignalSystem {
    fn new() -> Self {
        TradingSignalSystem {
            indicator_cache: HashMap::new(),
            signal_cache: HashMap::new(),
        }
    }

    fn get_signal(&mut self, symbol: &str, prices: &[f64]) -> (String, f64) {
        let timestamp = prices.len();  // Упрощённый "timestamp"
        let signal_key = format!("{}:{}", symbol, timestamp);

        // Проверяем кеш сигналов
        if let Some(cached) = self.signal_cache.get(&signal_key) {
            println!("[CACHE] Сигнал для {} из кеша", symbol);
            return cached.clone();
        }

        // Вычисляем индикаторы (с кешированием)
        let sma_20 = self.get_or_compute_indicator(
            &format!("{}:SMA:20:{}", symbol, timestamp),
            || Self::calculate_sma(prices, 20),
        );

        let sma_50 = self.get_or_compute_indicator(
            &format!("{}:SMA:50:{}", symbol, timestamp),
            || Self::calculate_sma(prices, 50),
        );

        let rsi = self.get_or_compute_indicator(
            &format!("{}:RSI:14:{}", symbol, timestamp),
            || Self::calculate_rsi(prices, 14),
        );

        // Генерируем сигнал
        let current_price = prices.last().copied().unwrap_or(0.0);
        let (signal, confidence) = self.generate_signal(current_price, sma_20, sma_50, rsi);

        // Кешируем сигнал
        self.signal_cache.insert(signal_key, (signal.clone(), confidence));

        (signal, confidence)
    }

    fn get_or_compute_indicator<F>(&mut self, key: &str, compute: F) -> f64
    where
        F: FnOnce() -> f64,
    {
        if let Some(&cached) = self.indicator_cache.get(key) {
            return cached;
        }

        let value = compute();
        self.indicator_cache.insert(key.to_string(), value);
        value
    }

    fn calculate_sma(prices: &[f64], period: usize) -> f64 {
        if prices.len() < period {
            return 0.0;
        }
        let slice = &prices[prices.len() - period..];
        slice.iter().sum::<f64>() / period as f64
    }

    fn calculate_rsi(prices: &[f64], period: usize) -> f64 {
        if prices.len() < period + 1 {
            return 50.0;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in (prices.len() - period)..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        100.0 - (100.0 / (1.0 + avg_gain / avg_loss))
    }

    fn generate_signal(&self, price: f64, sma_20: f64, sma_50: f64, rsi: f64) -> (String, f64) {
        let mut score = 0.0;

        // Сигнал по SMA
        if price > sma_20 && sma_20 > sma_50 {
            score += 0.4;  // Бычий тренд
        } else if price < sma_20 && sma_20 < sma_50 {
            score -= 0.4;  // Медвежий тренд
        }

        // Сигнал по RSI
        if rsi < 30.0 {
            score += 0.3;  // Перепроданность
        } else if rsi > 70.0 {
            score -= 0.3;  // Перекупленность
        }

        let (signal, confidence) = if score > 0.2 {
            ("BUY".to_string(), score.min(1.0))
        } else if score < -0.2 {
            ("SELL".to_string(), score.abs().min(1.0))
        } else {
            ("HOLD".to_string(), 1.0 - score.abs())
        };

        (signal, confidence)
    }

    fn cache_stats(&self) -> (usize, usize) {
        (self.indicator_cache.len(), self.signal_cache.len())
    }
}

fn main() {
    let mut system = TradingSignalSystem::new();

    // Генерируем исторические цены
    let prices: Vec<f64> = (0..100)
        .map(|i| 42000.0 + (i as f64 * 10.0).sin() * 500.0)
        .collect();

    // Запрашиваем сигнал несколько раз
    for i in 0..3 {
        println!("\nЗапрос #{}", i + 1);
        let (signal, confidence) = system.get_signal("BTC", &prices);
        println!("Сигнал: {} (уверенность: {:.2})", signal, confidence);
    }

    let (indicators, signals) = system.cache_stats();
    println!("\n=== Статистика кеша ===");
    println!("Индикаторов в кеше: {}", indicators);
    println!("Сигналов в кеше: {}", signals);
}
```

## Что мы узнали

| Концепция | Описание | Применение в трейдинге |
|-----------|----------|----------------------|
| Мемоизация | Кеширование результатов функций | Расчёт индикаторов |
| TTL кеш | Кеш с временем жизни | Рыночные данные |
| LRU кеш | Вытеснение по давности | Ограниченная память |
| Lazy Evaluation | Вычисление по требованию | Тяжёлые расчёты |
| Инвалидация | Сброс устаревших данных | Новые рыночные данные |

## Домашнее задание

1. Реализуй мемоизированный калькулятор MACD с кешированием EMA (экспоненциальной скользящей средней)

2. Создай систему кеширования свечных паттернов (Doji, Hammer, Engulfing) с TTL

3. Напиши LRU кеш для хранения последних N торговых решений с метаданными

4. Реализуй ленивую загрузку исторических данных с мемоизацией для бэктестинга

## Навигация

[← Предыдущий день](../149-data-streaming/ru.md) | [Следующий день →](../151-lazy-evaluation/ru.md)
