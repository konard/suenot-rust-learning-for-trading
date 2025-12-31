# День 281: Комиссии: учитываем издержки

## Аналогия из трейдинга

Представь, что ты решил продать свой старый телефон через сервис объявлений. Покупатель готов заплатить 10 000 рублей. Ты радуешься, но потом обнаруживаешь: сервис берёт 5% комиссии, банк — ещё 1% за перевод. В итоге на руках у тебя не 10 000, а 9 400 рублей. **Комиссии съели 6% прибыли!**

На бирже всё точно так же. Каждая сделка стоит денег:
- **Торговая комиссия биржи** — обычно 0.1-0.5% от объёма сделки
- **Комиссия мейкера/тейкера** — разные ставки за добавление/снятие ликвидности
- **Спред** — скрытая комиссия в разнице bid/ask
- **Комиссия за вывод** — фиксированная сумма за перевод средств

В бэктестинге мы **обязаны** учитывать эти издержки, иначе получим нереалистичные результаты. Стратегия, показывающая +20% прибыли без комиссий, может оказаться убыточной в реальности!

## Типы комиссий

```rust
/// Модель комиссий биржи
#[derive(Debug, Clone)]
pub struct CommissionModel {
    /// Процент комиссии для тейкера (снимает ликвидность)
    pub taker_fee_percent: f64,
    /// Процент комиссии для мейкера (добавляет ликвидность)
    pub maker_fee_percent: f64,
    /// Минимальная комиссия за сделку
    pub min_fee: f64,
    /// Максимальная комиссия за сделку (если есть ограничение)
    pub max_fee: Option<f64>,
}

impl CommissionModel {
    /// Создаёт модель с одинаковой комиссией для всех типов ордеров
    pub fn flat(fee_percent: f64) -> Self {
        CommissionModel {
            taker_fee_percent: fee_percent,
            maker_fee_percent: fee_percent,
            min_fee: 0.0,
            max_fee: None,
        }
    }

    /// Binance Spot комиссии (стандартный уровень)
    pub fn binance_spot() -> Self {
        CommissionModel {
            taker_fee_percent: 0.1,  // 0.1%
            maker_fee_percent: 0.1,  // 0.1%
            min_fee: 0.0,
            max_fee: None,
        }
    }

    /// Binance Futures комиссии
    pub fn binance_futures() -> Self {
        CommissionModel {
            taker_fee_percent: 0.04,  // 0.04%
            maker_fee_percent: 0.02,  // 0.02%
            min_fee: 0.0,
            max_fee: None,
        }
    }

    /// Bybit комиссии
    pub fn bybit() -> Self {
        CommissionModel {
            taker_fee_percent: 0.075,
            maker_fee_percent: 0.025,
            min_fee: 0.0,
            max_fee: None,
        }
    }

    /// Рассчитывает комиссию за сделку
    pub fn calculate(&self, trade_value: f64, is_taker: bool) -> f64 {
        let fee_percent = if is_taker {
            self.taker_fee_percent
        } else {
            self.maker_fee_percent
        };

        let fee = trade_value * (fee_percent / 100.0);

        // Применяем минимальную комиссию
        let fee = fee.max(self.min_fee);

        // Применяем максимальную комиссию, если она задана
        match self.max_fee {
            Some(max) => fee.min(max),
            None => fee,
        }
    }
}

fn main() {
    let binance = CommissionModel::binance_spot();

    // Покупаем BTC на $10,000
    let trade_value = 10_000.0;
    let commission = binance.calculate(trade_value, true);

    println!("Объём сделки: ${:.2}", trade_value);
    println!("Комиссия (тейкер): ${:.2}", commission);
    println!("Процент: {:.3}%", (commission / trade_value) * 100.0);
}
```

## Влияние комиссий на результаты

```rust
#[derive(Debug, Clone)]
pub struct Trade {
    pub symbol: String,
    pub side: TradeSide,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub is_taker_entry: bool,
    pub is_taker_exit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeSide {
    Long,
    Short,
}

impl Trade {
    /// Расчёт прибыли БЕЗ учёта комиссий
    pub fn gross_profit(&self) -> f64 {
        let price_diff = self.exit_price - self.entry_price;
        match self.side {
            TradeSide::Long => price_diff * self.quantity,
            TradeSide::Short => -price_diff * self.quantity,
        }
    }

    /// Расчёт прибыли С учётом комиссий
    pub fn net_profit(&self, commission: &CommissionModel) -> f64 {
        let gross = self.gross_profit();

        // Комиссия за вход
        let entry_value = self.entry_price * self.quantity;
        let entry_fee = commission.calculate(entry_value, self.is_taker_entry);

        // Комиссия за выход
        let exit_value = self.exit_price * self.quantity;
        let exit_fee = commission.calculate(exit_value, self.is_taker_exit);

        // Чистая прибыль = Gross - комиссии
        gross - entry_fee - exit_fee
    }

    /// Общая сумма комиссий за сделку
    pub fn total_commission(&self, commission: &CommissionModel) -> f64 {
        let entry_value = self.entry_price * self.quantity;
        let entry_fee = commission.calculate(entry_value, self.is_taker_entry);

        let exit_value = self.exit_price * self.quantity;
        let exit_fee = commission.calculate(exit_value, self.is_taker_exit);

        entry_fee + exit_fee
    }
}

fn main() {
    let commission = CommissionModel::binance_spot();

    let trade = Trade {
        symbol: "BTC/USDT".to_string(),
        side: TradeSide::Long,
        entry_price: 40_000.0,
        exit_price: 40_400.0,  // +1% движение
        quantity: 0.25,         // 0.25 BTC
        is_taker_entry: true,
        is_taker_exit: true,
    };

    let gross = trade.gross_profit();
    let net = trade.net_profit(&commission);
    let fees = trade.total_commission(&commission);

    println!("═══════════════════════════════════════");
    println!("           АНАЛИЗ СДЕЛКИ");
    println!("═══════════════════════════════════════");
    println!("Символ: {}", trade.symbol);
    println!("Направление: {:?}", trade.side);
    println!("Вход: ${:.2} x {:.4}", trade.entry_price, trade.quantity);
    println!("Выход: ${:.2} x {:.4}", trade.exit_price, trade.quantity);
    println!("───────────────────────────────────────");
    println!("Gross прибыль: ${:.2}", gross);
    println!("Комиссии:      ${:.2}", fees);
    println!("Net прибыль:   ${:.2}", net);
    println!("───────────────────────────────────────");
    println!("Комиссии съели: {:.1}% от gross прибыли", (fees / gross) * 100.0);
    println!("═══════════════════════════════════════");
}
```

## Трекер комиссий в бэктестере

```rust
use std::collections::HashMap;

/// Статистика комиссий за период
#[derive(Debug, Default, Clone)]
pub struct CommissionTracker {
    /// Общая сумма комиссий
    total_fees: f64,
    /// Комиссии по символам
    fees_by_symbol: HashMap<String, f64>,
    /// Количество сделок
    trade_count: u64,
    /// История комиссий для анализа
    fee_history: Vec<FeeRecord>,
}

#[derive(Debug, Clone)]
pub struct FeeRecord {
    pub timestamp: u64,
    pub symbol: String,
    pub trade_value: f64,
    pub fee_amount: f64,
    pub fee_type: FeeType,
}

#[derive(Debug, Clone, Copy)]
pub enum FeeType {
    Entry,
    Exit,
}

impl CommissionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Записывает комиссию
    pub fn record_fee(
        &mut self,
        timestamp: u64,
        symbol: &str,
        trade_value: f64,
        fee_amount: f64,
        fee_type: FeeType,
    ) {
        self.total_fees += fee_amount;
        *self.fees_by_symbol.entry(symbol.to_string()).or_default() += fee_amount;
        self.trade_count += 1;

        self.fee_history.push(FeeRecord {
            timestamp,
            symbol: symbol.to_string(),
            trade_value,
            fee_amount,
            fee_type,
        });
    }

    /// Общая сумма комиссий
    pub fn total_fees(&self) -> f64 {
        self.total_fees
    }

    /// Средняя комиссия за сделку
    pub fn average_fee(&self) -> f64 {
        if self.trade_count == 0 {
            0.0
        } else {
            self.total_fees / self.trade_count as f64
        }
    }

    /// Комиссии по символу
    pub fn fees_for_symbol(&self, symbol: &str) -> f64 {
        self.fees_by_symbol.get(symbol).copied().unwrap_or(0.0)
    }

    /// Топ-N символов по комиссиям
    pub fn top_symbols_by_fees(&self, n: usize) -> Vec<(String, f64)> {
        let mut sorted: Vec<_> = self.fees_by_symbol.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sorted.truncate(n);
        sorted
    }

    /// Отчёт по комиссиям
    pub fn print_report(&self) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║         ОТЧЁТ ПО КОМИССИЯМ            ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Всего комиссий:    ${:>16.2} ║", self.total_fees);
        println!("║ Количество сделок: {:>17} ║", self.trade_count);
        println!("║ Средняя комиссия:  ${:>16.2} ║", self.average_fee());
        println!("╠═══════════════════════════════════════╣");
        println!("║           ПО СИМВОЛАМ                 ║");
        println!("╠═══════════════════════════════════════╣");

        for (symbol, fee) in self.top_symbols_by_fees(5) {
            let percent = (fee / self.total_fees) * 100.0;
            println!("║ {:12} ${:>10.2} ({:>5.1}%) ║", symbol, fee, percent);
        }

        println!("╚═══════════════════════════════════════╝");
    }
}

fn main() {
    let mut tracker = CommissionTracker::new();

    // Симулируем несколько сделок
    tracker.record_fee(1, "BTC/USDT", 10_000.0, 10.0, FeeType::Entry);
    tracker.record_fee(2, "BTC/USDT", 10_100.0, 10.1, FeeType::Exit);
    tracker.record_fee(3, "ETH/USDT", 5_000.0, 5.0, FeeType::Entry);
    tracker.record_fee(4, "ETH/USDT", 5_050.0, 5.05, FeeType::Exit);
    tracker.record_fee(5, "SOL/USDT", 1_000.0, 1.0, FeeType::Entry);
    tracker.record_fee(6, "SOL/USDT", 950.0, 0.95, FeeType::Exit);

    tracker.print_report();
}
```

## Продвинутые модели комиссий

### Тиерные комиссии (в зависимости от объёма)

```rust
/// Уровень комиссий в зависимости от объёма торгов
#[derive(Debug, Clone)]
pub struct TieredCommission {
    tiers: Vec<CommissionTier>,
}

#[derive(Debug, Clone)]
pub struct CommissionTier {
    /// Минимальный объём для этого уровня (за 30 дней)
    pub min_volume: f64,
    /// Комиссия тейкера, %
    pub taker_fee: f64,
    /// Комиссия мейкера, %
    pub maker_fee: f64,
}

impl TieredCommission {
    /// Binance VIP уровни
    pub fn binance_vip() -> Self {
        TieredCommission {
            tiers: vec![
                CommissionTier { min_volume: 0.0, taker_fee: 0.1, maker_fee: 0.1 },
                CommissionTier { min_volume: 1_000_000.0, taker_fee: 0.09, maker_fee: 0.09 },
                CommissionTier { min_volume: 5_000_000.0, taker_fee: 0.08, maker_fee: 0.07 },
                CommissionTier { min_volume: 20_000_000.0, taker_fee: 0.07, maker_fee: 0.05 },
                CommissionTier { min_volume: 100_000_000.0, taker_fee: 0.06, maker_fee: 0.04 },
                CommissionTier { min_volume: 500_000_000.0, taker_fee: 0.05, maker_fee: 0.03 },
            ],
        }
    }

    /// Получает комиссию для заданного объёма
    pub fn get_commission(&self, monthly_volume: f64) -> CommissionModel {
        let tier = self.tiers.iter()
            .filter(|t| t.min_volume <= monthly_volume)
            .last()
            .unwrap();

        CommissionModel {
            taker_fee_percent: tier.taker_fee,
            maker_fee_percent: tier.maker_fee,
            min_fee: 0.0,
            max_fee: None,
        }
    }
}

fn main() {
    let tiered = TieredCommission::binance_vip();

    let volumes = [10_000.0, 500_000.0, 2_000_000.0, 50_000_000.0, 200_000_000.0];

    println!("Binance VIP уровни:");
    println!("────────────────────────────────────────────");
    println!("{:>15} | {:>10} | {:>10}", "Объём 30д", "Тейкер", "Мейкер");
    println!("────────────────────────────────────────────");

    for volume in volumes {
        let commission = tiered.get_commission(volume);
        println!(
            "${:>14.0} | {:>9.3}% | {:>9.3}%",
            volume, commission.taker_fee_percent, commission.maker_fee_percent
        );
    }
}
```

### Учёт скидок и кешбэков

```rust
/// Расширенная модель комиссий с учётом скидок
#[derive(Debug, Clone)]
pub struct AdvancedCommissionModel {
    base: CommissionModel,
    /// Скидка при оплате нативным токеном (например, BNB)
    native_token_discount: f64,
    /// Реферальная скидка
    referral_discount: f64,
    /// Кешбэк за объём
    volume_cashback_percent: f64,
}

impl AdvancedCommissionModel {
    pub fn new(base: CommissionModel) -> Self {
        AdvancedCommissionModel {
            base,
            native_token_discount: 0.0,
            referral_discount: 0.0,
            volume_cashback_percent: 0.0,
        }
    }

    /// Binance с BNB скидкой 25%
    pub fn binance_with_bnb() -> Self {
        AdvancedCommissionModel {
            base: CommissionModel::binance_spot(),
            native_token_discount: 25.0,  // 25% скидка
            referral_discount: 0.0,
            volume_cashback_percent: 0.0,
        }
    }

    /// Рассчитывает итоговую комиссию
    pub fn calculate(&self, trade_value: f64, is_taker: bool, use_native_token: bool) -> f64 {
        let base_fee = self.base.calculate(trade_value, is_taker);

        // Применяем скидки
        let mut discount_multiplier = 1.0;

        if use_native_token {
            discount_multiplier -= self.native_token_discount / 100.0;
        }

        discount_multiplier -= self.referral_discount / 100.0;

        let discounted_fee = base_fee * discount_multiplier;

        // Учитываем кешбэк (возвращается позже, но уменьшает реальную комиссию)
        let cashback = discounted_fee * (self.volume_cashback_percent / 100.0);

        discounted_fee - cashback
    }
}

fn main() {
    let model = AdvancedCommissionModel::binance_with_bnb();
    let trade_value = 10_000.0;

    let fee_without_bnb = model.calculate(trade_value, true, false);
    let fee_with_bnb = model.calculate(trade_value, true, true);

    println!("Объём сделки: ${:.2}", trade_value);
    println!("Комиссия без BNB: ${:.2}", fee_without_bnb);
    println!("Комиссия с BNB:   ${:.2}", fee_with_bnb);
    println!("Экономия:         ${:.2} ({:.1}%)",
        fee_without_bnb - fee_with_bnb,
        ((fee_without_bnb - fee_with_bnb) / fee_without_bnb) * 100.0
    );
}
```

## Интеграция с бэктестером

```rust
/// Результат бэктеста с учётом комиссий
#[derive(Debug)]
pub struct BacktestResult {
    pub gross_profit: f64,
    pub total_commissions: f64,
    pub net_profit: f64,
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub commission_tracker: CommissionTracker,
}

impl BacktestResult {
    pub fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            (self.winning_trades as f64 / self.total_trades as f64) * 100.0
        }
    }

    /// Процент прибыли, съеденной комиссиями
    pub fn commission_impact(&self) -> f64 {
        if self.gross_profit <= 0.0 {
            0.0
        } else {
            (self.total_commissions / self.gross_profit) * 100.0
        }
    }

    pub fn print_summary(&self) {
        println!("\n╔═══════════════════════════════════════════════╗");
        println!("║              РЕЗУЛЬТАТЫ БЭКТЕСТА              ║");
        println!("╠═══════════════════════════════════════════════╣");
        println!("║                   ПРИБЫЛЬ                     ║");
        println!("╟───────────────────────────────────────────────╢");
        println!("║ Gross прибыль:      {:>24.2} ║", self.gross_profit);
        println!("║ Комиссии:           {:>24.2} ║", self.total_commissions);
        println!("║ Net прибыль:        {:>24.2} ║", self.net_profit);
        println!("╟───────────────────────────────────────────────╢");
        println!("║                   СДЕЛКИ                      ║");
        println!("╟───────────────────────────────────────────────╢");
        println!("║ Всего сделок:       {:>24} ║", self.total_trades);
        println!("║ Прибыльных:         {:>24} ║", self.winning_trades);
        println!("║ Убыточных:          {:>24} ║", self.losing_trades);
        println!("║ Win Rate:           {:>23.1}% ║", self.win_rate());
        println!("╟───────────────────────────────────────────────╢");
        println!("║                   АНАЛИЗ                      ║");
        println!("╟───────────────────────────────────────────────╢");
        println!("║ Комиссии съели:     {:>23.1}% ║", self.commission_impact());
        println!("║ Ср. комиссия/сделка:{:>24.2} ║",
            self.commission_tracker.average_fee());
        println!("╚═══════════════════════════════════════════════╝");
    }
}

/// Простой бэктестер с учётом комиссий
pub struct SimpleBacktester {
    commission_model: CommissionModel,
    commission_tracker: CommissionTracker,
    trades: Vec<Trade>,
}

impl SimpleBacktester {
    pub fn new(commission_model: CommissionModel) -> Self {
        SimpleBacktester {
            commission_model,
            commission_tracker: CommissionTracker::new(),
            trades: Vec::new(),
        }
    }

    pub fn add_trade(&mut self, trade: Trade, timestamp: u64) {
        // Записываем комиссии
        let entry_value = trade.entry_price * trade.quantity;
        let entry_fee = self.commission_model.calculate(entry_value, trade.is_taker_entry);
        self.commission_tracker.record_fee(
            timestamp, &trade.symbol, entry_value, entry_fee, FeeType::Entry
        );

        let exit_value = trade.exit_price * trade.quantity;
        let exit_fee = self.commission_model.calculate(exit_value, trade.is_taker_exit);
        self.commission_tracker.record_fee(
            timestamp + 1, &trade.symbol, exit_value, exit_fee, FeeType::Exit
        );

        self.trades.push(trade);
    }

    pub fn run(&self) -> BacktestResult {
        let mut gross_profit = 0.0;
        let mut net_profit = 0.0;
        let mut winning_trades = 0u64;
        let mut losing_trades = 0u64;

        for trade in &self.trades {
            let gross = trade.gross_profit();
            let net = trade.net_profit(&self.commission_model);

            gross_profit += gross;
            net_profit += net;

            if net > 0.0 {
                winning_trades += 1;
            } else {
                losing_trades += 1;
            }
        }

        BacktestResult {
            gross_profit,
            total_commissions: self.commission_tracker.total_fees(),
            net_profit,
            total_trades: self.trades.len() as u64,
            winning_trades,
            losing_trades,
            commission_tracker: self.commission_tracker.clone(),
        }
    }
}

fn main() {
    let commission = CommissionModel::binance_spot();
    let mut backtester = SimpleBacktester::new(commission);

    // Добавляем тестовые сделки
    let trades = vec![
        Trade {
            symbol: "BTC/USDT".to_string(),
            side: TradeSide::Long,
            entry_price: 40_000.0,
            exit_price: 40_800.0,  // +2%
            quantity: 0.5,
            is_taker_entry: true,
            is_taker_exit: true,
        },
        Trade {
            symbol: "ETH/USDT".to_string(),
            side: TradeSide::Long,
            entry_price: 2_500.0,
            exit_price: 2_450.0,  // -2%
            quantity: 2.0,
            is_taker_entry: true,
            is_taker_exit: true,
        },
        Trade {
            symbol: "BTC/USDT".to_string(),
            side: TradeSide::Short,
            entry_price: 41_000.0,
            exit_price: 40_500.0,  // +1.2% для шорта
            quantity: 0.3,
            is_taker_entry: false,  // лимитный ордер = мейкер
            is_taker_exit: true,
        },
        Trade {
            symbol: "SOL/USDT".to_string(),
            side: TradeSide::Long,
            entry_price: 100.0,
            exit_price: 105.0,  // +5%
            quantity: 50.0,
            is_taker_entry: true,
            is_taker_exit: false,  // лимитный ордер = мейкер
        },
    ];

    for (i, trade) in trades.into_iter().enumerate() {
        backtester.add_trade(trade, i as u64 * 100);
    }

    let result = backtester.run();
    result.print_summary();
    result.commission_tracker.print_report();
}
```

## Сравнение стратегий с учётом комиссий

```rust
/// Анализ чувствительности стратегии к комиссиям
pub fn analyze_commission_sensitivity(
    base_gross_profit: f64,
    trade_count: u64,
    avg_trade_value: f64,
) {
    println!("\n═══════════════════════════════════════════════════════════");
    println!("      АНАЛИЗ ЧУВСТВИТЕЛЬНОСТИ К КОМИССИЯМ");
    println!("═══════════════════════════════════════════════════════════");
    println!("Gross прибыль: ${:.2}", base_gross_profit);
    println!("Количество сделок: {}", trade_count);
    println!("Средний объём сделки: ${:.2}", avg_trade_value);
    println!("───────────────────────────────────────────────────────────");
    println!("{:>10} | {:>15} | {:>15} | {:>12}",
        "Комиссия", "Всего комиссий", "Net прибыль", "% от Gross");
    println!("───────────────────────────────────────────────────────────");

    let commission_rates = [0.01, 0.05, 0.1, 0.15, 0.2, 0.3, 0.5];

    for rate in commission_rates {
        // Комиссия за вход и выход
        let total_commission = avg_trade_value * (rate / 100.0) * 2.0 * trade_count as f64;
        let net_profit = base_gross_profit - total_commission;
        let impact = (total_commission / base_gross_profit) * 100.0;

        let status = if net_profit > 0.0 { "+" } else { "" };

        println!(
            "{:>9.2}% | ${:>14.2} | {:>1}${:>13.2} | {:>11.1}%",
            rate, total_commission, status, net_profit.abs(), impact
        );
    }
    println!("═══════════════════════════════════════════════════════════");
}

fn main() {
    // Стратегия A: много мелких сделок
    println!("\nСтратегия A: Скальпинг (много мелких сделок)");
    analyze_commission_sensitivity(
        5000.0,      // Gross прибыль
        500,         // 500 сделок
        1000.0,      // Средний объём $1000
    );

    // Стратегия B: мало крупных сделок
    println!("\nСтратегия B: Свинг-трейдинг (мало крупных сделок)");
    analyze_commission_sensitivity(
        5000.0,      // Такая же gross прибыль
        20,          // Всего 20 сделок
        10000.0,     // Средний объём $10,000
    );
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Торговые комиссии | Биржи берут процент от объёма сделки |
| Тейкер/Мейкер | Разные ставки за снятие/добавление ликвидности |
| Gross vs Net | Прибыль до и после вычета комиссий |
| Тиерные комиссии | Скидки за большой объём торгов |
| Влияние частоты | Чем больше сделок, тем больше комиссий |
| Бэктестинг | Без учёта комиссий результаты нереалистичны |

## Домашнее задание

1. **Калькулятор комиссий**: Создай функцию `compare_exchanges(trade_value: f64, monthly_volume: f64)`, которая сравнивает комиссии разных бирж (Binance, Bybit, OKX) с учётом тиерных скидок и выводит самый выгодный вариант.

2. **Оптимальная частота сделок**: Напиши программу, которая для заданной стратегии с фиксированным ожидаемым доходом вычисляет оптимальное количество сделок в месяц, при котором Net прибыль максимальна.

3. **Трекер издержек**: Расширь `CommissionTracker`, добавив:
   - Отслеживание комиссий по дням/неделям/месяцам
   - Расчёт среднедневных комиссий
   - Предупреждение, когда комиссии превышают заданный порог от прибыли

4. **Сравнение стратегий**: Имея две стратегии с одинаковой Gross прибылью, но разным количеством сделок, рассчитай, при какой комиссии биржи они становятся одинаково выгодными.

## Навигация

[← Предыдущий день](../280-slippage-model/ru.md) | [Следующий день →](../282-equity-curve/ru.md)
