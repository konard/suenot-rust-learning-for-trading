# День 256: Паттерн: индикатор как trait

## Аналогия из трейдинга

Представь, что ты строишь торговый дашборд, который должен отображать различные технические индикаторы — SMA, EMA, RSI, MACD, полосы Боллинджера. Каждый индикатор принимает ценовые данные и выдаёт какой-то результат, но делает это по-разному. Некоторые вычисляют одно значение, другие создают несколько линий, а третьи генерируют сигналы.

В торговом терминале ты хочешь иметь унифицированный способ:
- **Обновить** любой индикатор новыми ценовыми данными
- **Получить текущее значение** от любого индикатора
- **Сбросить** любой индикатор в начальное состояние

Именно это и обеспечивают трейты в Rust: **контракт**, которому должны следовать различные типы индикаторов. Подобно тому как все биржи имеют стандартный API для размещения ордеров (даже если внутренняя реализация отличается), все индикаторы могут использовать общий интерфейс через трейты.

## Что такое Trait?

Трейт определяет общее поведение, которое могут реализовывать типы. Думай о нём как о "контракте возможностей" — любой тип, реализующий трейт, гарантирует, что может выполнять определённые операции.

```rust
// Определяем, что ЛЮБОЙ индикатор должен уметь делать
trait Indicator {
    fn update(&mut self, price: f64);
    fn value(&self) -> Option<f64>;
    fn reset(&mut self);
}
```

Это говорит: "Любой тип, который заявляет себя как Indicator, ДОЛЖЕН предоставить эти три метода."

## Базовая реализация трейта индикатора

Реализуем наш трейт `Indicator` для простой скользящей средней (SMA):

```rust
trait Indicator {
    /// Передать новую цену индикатору
    fn update(&mut self, price: f64);

    /// Получить текущее значение индикатора (None если недостаточно данных)
    fn value(&self) -> Option<f64>;

    /// Сбросить индикатор в начальное состояние
    fn reset(&mut self);

    /// Получить название индикатора
    fn name(&self) -> &str;
}

struct SMA {
    period: usize,
    prices: Vec<f64>,
}

impl SMA {
    fn new(period: usize) -> Self {
        SMA {
            period,
            prices: Vec::with_capacity(period),
        }
    }
}

impl Indicator for SMA {
    fn update(&mut self, price: f64) {
        self.prices.push(price);
        if self.prices.len() > self.period {
            self.prices.remove(0);
        }
    }

    fn value(&self) -> Option<f64> {
        if self.prices.len() < self.period {
            return None;
        }
        let sum: f64 = self.prices.iter().sum();
        Some(sum / self.period as f64)
    }

    fn reset(&mut self) {
        self.prices.clear();
    }

    fn name(&self) -> &str {
        "SMA"
    }
}

fn main() {
    let mut sma = SMA::new(3);

    // Подаём цены
    for price in [100.0, 102.0, 104.0, 103.0, 105.0] {
        sma.update(price);
        match sma.value() {
            Some(v) => println!("{}: {:.2}", sma.name(), v),
            None => println!("{}: Вычисляется...", sma.name()),
        }
    }
}
```

Вывод:
```
SMA: Вычисляется...
SMA: Вычисляется...
SMA: 102.00
SMA: 103.00
SMA: 104.00
```

## Несколько индикаторов с одним трейтом

Теперь добавим EMA (экспоненциальную скользящую среднюю), используя тот же трейт:

```rust
struct EMA {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    count: usize,
    initial_sum: f64,
}

impl EMA {
    fn new(period: usize) -> Self {
        let multiplier = 2.0 / (period as f64 + 1.0);
        EMA {
            period,
            multiplier,
            current_ema: None,
            count: 0,
            initial_sum: 0.0,
        }
    }
}

impl Indicator for EMA {
    fn update(&mut self, price: f64) {
        self.count += 1;

        match self.current_ema {
            None => {
                // Накапливаем цены для начального SMA
                self.initial_sum += price;
                if self.count >= self.period {
                    // Используем SMA как первое значение EMA
                    self.current_ema = Some(self.initial_sum / self.period as f64);
                }
            }
            Some(prev_ema) => {
                // Формула EMA: EMA = (Цена - Предыдущий EMA) * Множитель + Предыдущий EMA
                let new_ema = (price - prev_ema) * self.multiplier + prev_ema;
                self.current_ema = Some(new_ema);
            }
        }
    }

    fn value(&self) -> Option<f64> {
        self.current_ema
    }

    fn reset(&mut self) {
        self.current_ema = None;
        self.count = 0;
        self.initial_sum = 0.0;
    }

    fn name(&self) -> &str {
        "EMA"
    }
}

fn main() {
    let mut sma = SMA::new(3);
    let mut ema = EMA::new(3);

    let prices = [100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 106.0];

    println!("{:<6} {:>10} {:>10}", "Цена", "SMA(3)", "EMA(3)");
    println!("{}", "-".repeat(28));

    for price in prices {
        sma.update(price);
        ema.update(price);

        let sma_val = sma.value().map(|v| format!("{:.2}", v)).unwrap_or("-".to_string());
        let ema_val = ema.value().map(|v| format!("{:.2}", v)).unwrap_or("-".to_string());

        println!("{:<6.1} {:>10} {:>10}", price, sma_val, ema_val);
    }
}
```

## Сила трейт-объектов

С трейтами мы можем хранить разные типы индикаторов в одной коллекции, используя **трейт-объекты**:

```rust
fn main() {
    // Храним разные индикаторы в одном векторе, используя трейт-объекты
    let mut indicators: Vec<Box<dyn Indicator>> = vec![
        Box::new(SMA::new(3)),
        Box::new(SMA::new(5)),
        Box::new(EMA::new(3)),
        Box::new(EMA::new(5)),
    ];

    let prices = [100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 106.0, 108.0];

    for price in prices {
        println!("\nЦена: {:.1}", price);
        for indicator in &mut indicators {
            indicator.update(price);
            if let Some(value) = indicator.value() {
                println!("  {}: {:.2}", indicator.name(), value);
            }
        }
    }
}
```

Это невероятно мощно — мы можем добавлять новые типы индикаторов без изменения кода обработки!

## Реализация RSI

Реализуем более сложный индикатор — RSI (индекс относительной силы):

```rust
struct RSI {
    period: usize,
    gains: Vec<f64>,
    losses: Vec<f64>,
    prev_price: Option<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
}

impl RSI {
    fn new(period: usize) -> Self {
        RSI {
            period,
            gains: Vec::new(),
            losses: Vec::new(),
            prev_price: None,
            avg_gain: None,
            avg_loss: None,
        }
    }
}

impl Indicator for RSI {
    fn update(&mut self, price: f64) {
        if let Some(prev) = self.prev_price {
            let change = price - prev;
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { -change } else { 0.0 };

            match (&self.avg_gain, &self.avg_loss) {
                (None, None) => {
                    // Начальный период: собираем приросты и потери
                    self.gains.push(gain);
                    self.losses.push(loss);

                    if self.gains.len() >= self.period {
                        self.avg_gain = Some(self.gains.iter().sum::<f64>() / self.period as f64);
                        self.avg_loss = Some(self.losses.iter().sum::<f64>() / self.period as f64);
                    }
                }
                (Some(prev_avg_gain), Some(prev_avg_loss)) => {
                    // Сглаженные средние
                    let new_avg_gain = (prev_avg_gain * (self.period - 1) as f64 + gain) / self.period as f64;
                    let new_avg_loss = (prev_avg_loss * (self.period - 1) as f64 + loss) / self.period as f64;
                    self.avg_gain = Some(new_avg_gain);
                    self.avg_loss = Some(new_avg_loss);
                }
                _ => unreachable!(),
            }
        }
        self.prev_price = Some(price);
    }

    fn value(&self) -> Option<f64> {
        match (&self.avg_gain, &self.avg_loss) {
            (Some(avg_gain), Some(avg_loss)) => {
                if *avg_loss == 0.0 {
                    Some(100.0) // Нет потерь означает RSI = 100
                } else {
                    let rs = avg_gain / avg_loss;
                    Some(100.0 - (100.0 / (1.0 + rs)))
                }
            }
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.gains.clear();
        self.losses.clear();
        self.prev_price = None;
        self.avg_gain = None;
        self.avg_loss = None;
    }

    fn name(&self) -> &str {
        "RSI"
    }
}
```

## Методы по умолчанию в трейтах

Трейты могут предоставлять реализации по умолчанию, которые можно переопределить:

```rust
trait Indicator {
    fn update(&mut self, price: f64);
    fn value(&self) -> Option<f64>;
    fn reset(&mut self);
    fn name(&self) -> &str;

    /// Реализация по умолчанию: проверить готов ли индикатор
    fn is_ready(&self) -> bool {
        self.value().is_some()
    }

    /// Реализация по умолчанию: получить форматированное значение
    fn formatted_value(&self) -> String {
        match self.value() {
            Some(v) => format!("{}: {:.2}", self.name(), v),
            None => format!("{}: Н/Д", self.name()),
        }
    }

    /// Реализация по умолчанию: обновить несколькими ценами
    fn update_batch(&mut self, prices: &[f64]) {
        for &price in prices {
            self.update(price);
        }
    }
}
```

Теперь все индикаторы автоматически получают эти методы:

```rust
fn main() {
    let mut rsi = RSI::new(14);

    // Используем метод update_batch по умолчанию
    rsi.update_batch(&[100.0, 101.0, 99.0, 102.0, 103.0, 101.0, 104.0,
                       105.0, 103.0, 106.0, 107.0, 105.0, 108.0, 109.0, 110.0]);

    // Используем метод is_ready по умолчанию
    if rsi.is_ready() {
        println!("{}", rsi.formatted_value());
    }
}
```

## Ассоциированные типы в трейтах

Для индикаторов, которые возвращают разные типы результатов, используем ассоциированные типы:

```rust
trait AdvancedIndicator {
    type Output;

    fn update(&mut self, price: f64);
    fn value(&self) -> Option<Self::Output>;
    fn name(&self) -> &str;
}

// Полосы Боллинджера возвращают три значения
#[derive(Debug, Clone)]
struct BollingerBandsOutput {
    upper: f64,
    middle: f64,
    lower: f64,
}

struct BollingerBands {
    period: usize,
    std_dev_multiplier: f64,
    prices: Vec<f64>,
}

impl BollingerBands {
    fn new(period: usize, std_dev_multiplier: f64) -> Self {
        BollingerBands {
            period,
            std_dev_multiplier,
            prices: Vec::with_capacity(period),
        }
    }
}

impl AdvancedIndicator for BollingerBands {
    type Output = BollingerBandsOutput;

    fn update(&mut self, price: f64) {
        self.prices.push(price);
        if self.prices.len() > self.period {
            self.prices.remove(0);
        }
    }

    fn value(&self) -> Option<Self::Output> {
        if self.prices.len() < self.period {
            return None;
        }

        let sum: f64 = self.prices.iter().sum();
        let mean = sum / self.period as f64;

        let variance: f64 = self.prices.iter()
            .map(|&p| (p - mean).powi(2))
            .sum::<f64>() / self.period as f64;
        let std_dev = variance.sqrt();

        Some(BollingerBandsOutput {
            upper: mean + self.std_dev_multiplier * std_dev,
            middle: mean,
            lower: mean - self.std_dev_multiplier * std_dev,
        })
    }

    fn name(&self) -> &str {
        "Полосы Боллинджера"
    }
}

fn main() {
    let mut bb = BollingerBands::new(20, 2.0);

    let prices: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 0.5) + (i % 3) as f64).collect();

    for price in &prices {
        bb.update(*price);
    }

    if let Some(bands) = bb.value() {
        println!("{}", bb.name());
        println!("  Верхняя:  {:.2}", bands.upper);
        println!("  Средняя:  {:.2}", bands.middle);
        println!("  Нижняя:   {:.2}", bands.lower);
    }
}
```

## Создание движка индикаторов

Создадим полноценный движок индикаторов с использованием трейтов:

```rust
use std::collections::HashMap;

trait Indicator {
    fn update(&mut self, price: f64);
    fn value(&self) -> Option<f64>;
    fn reset(&mut self);
    fn name(&self) -> &str;
}

struct IndicatorEngine {
    indicators: HashMap<String, Box<dyn Indicator>>,
    last_price: Option<f64>,
}

impl IndicatorEngine {
    fn new() -> Self {
        IndicatorEngine {
            indicators: HashMap::new(),
            last_price: None,
        }
    }

    fn add_indicator(&mut self, id: &str, indicator: Box<dyn Indicator>) {
        self.indicators.insert(id.to_string(), indicator);
    }

    fn remove_indicator(&mut self, id: &str) -> Option<Box<dyn Indicator>> {
        self.indicators.remove(id)
    }

    fn update(&mut self, price: f64) {
        self.last_price = Some(price);
        for indicator in self.indicators.values_mut() {
            indicator.update(price);
        }
    }

    fn get_value(&self, id: &str) -> Option<f64> {
        self.indicators.get(id)?.value()
    }

    fn get_all_values(&self) -> HashMap<String, Option<f64>> {
        self.indicators
            .iter()
            .map(|(id, ind)| (id.clone(), ind.value()))
            .collect()
    }

    fn reset_all(&mut self) {
        for indicator in self.indicators.values_mut() {
            indicator.reset();
        }
        self.last_price = None;
    }

    fn print_status(&self) {
        println!("\n=== Статус индикаторов ===");
        if let Some(price) = self.last_price {
            println!("Последняя цена: {:.2}", price);
        }
        for (id, indicator) in &self.indicators {
            let value_str = match indicator.value() {
                Some(v) => format!("{:.2}", v),
                None => "Н/Д".to_string(),
            };
            println!("{} ({}): {}", id, indicator.name(), value_str);
        }
    }
}

fn main() {
    let mut engine = IndicatorEngine::new();

    // Добавляем различные индикаторы
    engine.add_indicator("fast_sma", Box::new(SMA::new(5)));
    engine.add_indicator("slow_sma", Box::new(SMA::new(20)));
    engine.add_indicator("fast_ema", Box::new(EMA::new(5)));
    engine.add_indicator("slow_ema", Box::new(EMA::new(20)));
    engine.add_indicator("rsi_14", Box::new(RSI::new(14)));

    // Симулируем ценовые данные
    let prices = [
        100.0, 101.5, 103.0, 102.5, 104.0, 105.5, 104.5, 106.0,
        107.5, 108.0, 107.0, 109.0, 110.5, 109.5, 111.0, 112.0,
        111.5, 113.0, 114.5, 113.5, 115.0, 116.5, 115.5, 117.0,
        118.0, 117.5, 119.0, 120.0, 119.0, 121.0,
    ];

    for &price in &prices {
        engine.update(price);
    }

    engine.print_status();

    // Проверяем сигнал пересечения SMA
    let fast = engine.get_value("fast_sma");
    let slow = engine.get_value("slow_sma");

    if let (Some(f), Some(s)) = (fast, slow) {
        if f > s {
            println!("\nСигнал: БЫЧИЙ (Быстрая SMA выше Медленной SMA)");
        } else {
            println!("\nСигнал: МЕДВЕЖИЙ (Быстрая SMA ниже Медленной SMA)");
        }
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Трейты | Определение общего поведения, которое могут реализовывать типы |
| `impl Trait for Type` | Реализация трейта для конкретного типа |
| Трейт-объекты (`dyn Trait`) | Полиморфизм во время выполнения для разных типов |
| `Box<dyn Trait>` | Размещённые в куче трейт-объекты для коллекций |
| Методы по умолчанию | Методы трейта с реализацией по умолчанию |
| Ассоциированные типы | Заполнители типов внутри трейтов |

## Упражнения

1. **Индикатор MACD**: Реализуй индикатор MACD, используя трейт `Indicator`. MACD состоит из:
   - Линия MACD = 12-периодный EMA - 26-периодный EMA
   - Сигнальная линия = 9-периодный EMA от линии MACD
   - Гистограмма = Линия MACD - Сигнальная линия

   Подсказка: Можно использовать уже существующие индикаторы EMA!

2. **Стохастический осциллятор**: Реализуй стохастический индикатор:
   - %K = (Текущее закрытие - Минимальный минимум) / (Максимальный максимум - Минимальный минимум) * 100
   - %D = 3-периодный SMA от %K

3. **ATR (Средний истинный диапазон)**: Реализуй ATR, который вычисляет:
   - True Range = max(High - Low, |High - Предыдущее закрытие|, |Low - Предыдущее закрытие|)
   - ATR = Скользящая средняя от True Range

4. **Трейт генератора сигналов**: Создай трейт `SignalGenerator`, который работает с индикаторами:
   ```rust
   trait SignalGenerator {
       fn generate_signal(&self) -> Signal;
   }

   enum Signal {
       Buy,
       Sell,
       Hold,
   }
   ```

## Домашнее задание

1. **Фабрика индикаторов**: Создай `IndicatorFactory`, которая может создавать индикаторы по имени:
   ```rust
   let sma = factory.create("SMA", &[("period", 20)]);
   let ema = factory.create("EMA", &[("period", 12)]);
   ```

2. **Составной индикатор**: Создай `CompositeIndicator`, который объединяет несколько индикаторов с пользовательской логикой:
   ```rust
   let composite = CompositeIndicator::new()
       .add(SMA::new(10))
       .add(EMA::new(10))
       .combine(|values| values.iter().sum::<f64>() / values.len() as f64);
   ```

3. **Сериализуемые индикаторы**: Добавь поддержку serde для сохранения/загрузки состояния индикатора:
   ```rust
   let json = indicator.to_json();
   let restored: Box<dyn Indicator> = Indicator::from_json(&json);
   ```

4. **Бэктестер индикаторов**: Создай простой бэктестер для тестирования стратегий на основе индикаторов:
   ```rust
   let strategy = CrossoverStrategy::new(SMA::new(10), SMA::new(50));
   let results = backtest(strategy, &historical_prices);
   println!("Общая доходность: {:.2}%", results.total_return);
   ```

## Навигация

[← Предыдущий день](../255-vwap-volume-weighted-price/ru.md) | [Следующий день →](../257-combining-indicators/ru.md)
