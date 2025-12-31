# День 301: Paper trading: виртуальная торговля

## Аналогия из трейдинга

Представь начинающего трейдера, который хочет протестировать новую стратегию, но не хочет рисковать реальными деньгами. Он открывает демо-счёт с виртуальными средствами и начинает совершать сделки. Все операции выглядят реально: открытие позиций, стоп-лоссы, тейк-профиты, но деньги виртуальные. Это и есть **paper trading** — практика без финансового риска.

Как пилот, тренирующийся на симуляторе перед полётом на настоящем самолёте:
- Все инструменты работают реалистично
- Можно отработать аварийные ситуации
- Ошибки не стоят жизней (или денег)
- Можно повторять одну и ту же ситуацию много раз
- Вырабатывается уверенность перед реальной торговлей

В алготрейдинге paper trading — критически важный этап между бэктестингом на исторических данных и живой торговлей на реальные деньги.

## Зачем нужен Paper Trading

| Этап | Цель | Уровень риска |
|------|------|---------------|
| **Бэктестинг** | Тест на исторических данных | Нет риска (прошлые данные) |
| **Paper Trading** | Тест в реальном времени | Нет финансового риска |
| **Живая торговля** | Реальные деньги, реальные сделки | Полный финансовый риск |

Paper trading позволяет:
1. Протестировать стратегию в реальных рыночных условиях
2. Отладить код в продакшн-подобном окружении
3. Отработать исполнение и управление ордерами
4. Протестировать интеграцию с API без риска
5. Выработать психологическую готовность к реальной торговле
6. Проверить производительность стратегии за пределами бэктестов

## Базовый Paper Trading счёт

```rust
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct PaperAccount {
    balance: f64,
    initial_balance: f64,
    positions: HashMap<String, Position>,
    trade_history: Vec<Trade>,
    commission_rate: f64, // Комиссия в процентах
}

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    entry_time: u64,
    side: Side,
}

#[derive(Debug, Clone, PartialEq)]
enum Side {
    Long,
    Short,
}

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    side: Side,
    quantity: f64,
    entry_price: f64,
    exit_price: f64,
    entry_time: u64,
    exit_time: u64,
    pnl: f64,
    commission: f64,
}

impl PaperAccount {
    fn new(initial_balance: f64, commission_rate: f64) -> Self {
        Self {
            balance: initial_balance,
            initial_balance,
            positions: HashMap::new(),
            trade_history: Vec::new(),
            commission_rate,
        }
    }

    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Открыть длинную позицию
    fn open_long(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<String, String> {
        if self.positions.contains_key(symbol) {
            return Err(format!("Позиция по {} уже существует", symbol));
        }

        let cost = quantity * price;
        let commission = cost * self.commission_rate;
        let total_cost = cost + commission;

        if total_cost > self.balance {
            return Err(format!(
                "Недостаточно средств: нужно {:.2}, есть {:.2}",
                total_cost, self.balance
            ));
        }

        self.balance -= total_cost;

        self.positions.insert(
            symbol.to_string(),
            Position {
                symbol: symbol.to_string(),
                quantity,
                entry_price: price,
                entry_time: Self::get_timestamp(),
                side: Side::Long,
            },
        );

        Ok(format!(
            "Открыт LONG {} @ {:.2} (кол-во: {}, стоимость: {:.2}, комиссия: {:.2})",
            symbol, price, quantity, cost, commission
        ))
    }

    /// Открыть короткую позицию
    fn open_short(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<String, String> {
        if self.positions.contains_key(symbol) {
            return Err(format!("Позиция по {} уже существует", symbol));
        }

        let proceeds = quantity * price;
        let commission = proceeds * self.commission_rate;

        // Для шорта мы получаем выручку, но платим комиссию
        self.balance += proceeds - commission;

        self.positions.insert(
            symbol.to_string(),
            Position {
                symbol: symbol.to_string(),
                quantity,
                entry_price: price,
                entry_time: Self::get_timestamp(),
                side: Side::Short,
            },
        );

        Ok(format!(
            "Открыт SHORT {} @ {:.2} (кол-во: {}, выручка: {:.2}, комиссия: {:.2})",
            symbol, price, quantity, proceeds, commission
        ))
    }

    /// Закрыть позицию
    fn close_position(&mut self, symbol: &str, exit_price: f64) -> Result<String, String> {
        let position = self
            .positions
            .remove(symbol)
            .ok_or(format!("Нет позиции по {}", symbol))?;

        let exit_time = Self::get_timestamp();

        let (pnl, commission) = match position.side {
            Side::Long => {
                // Лонг: прибыль когда цена растёт
                let proceeds = position.quantity * exit_price;
                let commission = proceeds * self.commission_rate;
                let pnl = proceeds - (position.quantity * position.entry_price);
                self.balance += proceeds - commission;
                (pnl, commission)
            }
            Side::Short => {
                // Шорт: прибыль когда цена падает
                let cost = position.quantity * exit_price;
                let commission = cost * self.commission_rate;
                let pnl = (position.quantity * position.entry_price) - cost;
                self.balance -= cost + commission;
                (pnl, commission)
            }
        };

        let trade = Trade {
            symbol: symbol.to_string(),
            side: position.side.clone(),
            quantity: position.quantity,
            entry_price: position.entry_price,
            exit_price,
            entry_time: position.entry_time,
            exit_time,
            pnl,
            commission,
        };

        self.trade_history.push(trade.clone());

        Ok(format!(
            "Закрыт {} {} @ {:.2} (вход: {:.2}, PnL: {:.2}, комиссия: {:.2})",
            if position.side == Side::Long {
                "LONG"
            } else {
                "SHORT"
            },
            symbol,
            exit_price,
            position.entry_price,
            pnl,
            commission
        ))
    }

    /// Получить текущий нереализованный PnL для всех позиций
    fn get_unrealized_pnl(&self, current_prices: &HashMap<String, f64>) -> f64 {
        self.positions
            .iter()
            .map(|(symbol, position)| {
                if let Some(&current_price) = current_prices.get(symbol) {
                    match position.side {
                        Side::Long => {
                            position.quantity * (current_price - position.entry_price)
                        }
                        Side::Short => {
                            position.quantity * (position.entry_price - current_price)
                        }
                    }
                } else {
                    0.0
                }
            })
            .sum()
    }

    /// Получить общую стоимость счёта (баланс + нереализованный PnL)
    fn get_total_value(&self, current_prices: &HashMap<String, f64>) -> f64 {
        self.balance + self.get_unrealized_pnl(current_prices)
    }

    /// Получить статистику по счёту
    fn get_stats(&self) -> String {
        if self.trade_history.is_empty() {
            return "Сделок ещё не было".to_string();
        }

        let total_trades = self.trade_history.len();
        let winning_trades = self
            .trade_history
            .iter()
            .filter(|t| t.pnl > 0.0)
            .count();
        let losing_trades = total_trades - winning_trades;

        let total_pnl: f64 = self.trade_history.iter().map(|t| t.pnl).sum();
        let total_commission: f64 = self.trade_history.iter().map(|t| t.commission).sum();
        let net_pnl = total_pnl - total_commission;

        let win_rate = (winning_trades as f64 / total_trades as f64) * 100.0;
        let roi = (net_pnl / self.initial_balance) * 100.0;

        format!(
            "=== Статистика Paper Trading ===\n\
             Начальный баланс: ${:.2}\n\
             Текущий баланс: ${:.2}\n\
             Всего сделок: {}\n\
             Прибыльных сделок: {} ({:.1}%)\n\
             Убыточных сделок: {}\n\
             Общий PnL: ${:.2}\n\
             Общая комиссия: ${:.2}\n\
             Чистый PnL: ${:.2}\n\
             ROI: {:.2}%",
            self.initial_balance,
            self.balance,
            total_trades,
            winning_trades,
            win_rate,
            losing_trades,
            total_pnl,
            total_commission,
            net_pnl,
            roi
        )
    }
}

fn main() {
    let mut account = PaperAccount::new(10000.0, 0.001); // $10,000 с комиссией 0.1%

    println!("=== Начинаем Paper Trading ===");
    println!("Начальный баланс: ${:.2}\n", account.balance);

    // Симулируем несколько сделок
    println!("{}", account.open_long("BTC", 0.5, 42000.0).unwrap());
    println!("{}", account.open_short("ETH", 10.0, 2500.0).unwrap());
    println!();

    // Проверяем позиции
    let mut current_prices = HashMap::new();
    current_prices.insert("BTC".to_string(), 43000.0);
    current_prices.insert("ETH".to_string(), 2450.0);

    println!("Текущий баланс: ${:.2}", account.balance);
    println!(
        "Нереализованный PnL: ${:.2}",
        account.get_unrealized_pnl(&current_prices)
    );
    println!(
        "Общая стоимость счёта: ${:.2}\n",
        account.get_total_value(&current_prices)
    );

    // Закрываем позиции
    println!("{}", account.close_position("BTC", 43000.0).unwrap());
    println!("{}", account.close_position("ETH", 2450.0).unwrap());
    println!();

    // Показываем статистику
    println!("{}", account.get_stats());
}
```

## Система управления ордерами

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

#[derive(Debug, Clone, PartialEq)]
enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: Side,
    order_type: OrderType,
    quantity: f64,
    price: Option<f64>, // Для лимитных/стоп ордеров
    status: OrderStatus,
    created_at: u64,
    filled_at: Option<u64>,
}

struct OrderBook {
    next_order_id: u64,
    pending_orders: VecDeque<Order>,
    filled_orders: Vec<Order>,
}

impl OrderBook {
    fn new() -> Self {
        Self {
            next_order_id: 1,
            pending_orders: VecDeque::new(),
            filled_orders: Vec::new(),
        }
    }

    fn place_order(
        &mut self,
        symbol: &str,
        side: Side,
        order_type: OrderType,
        quantity: f64,
        price: Option<f64>,
    ) -> u64 {
        let order = Order {
            id: self.next_order_id,
            symbol: symbol.to_string(),
            side,
            order_type,
            quantity,
            price,
            status: OrderStatus::Pending,
            created_at: PaperAccount::get_timestamp(),
            filled_at: None,
        };

        self.next_order_id += 1;
        self.pending_orders.push_back(order);
        self.next_order_id - 1
    }

    fn process_orders(
        &mut self,
        account: &mut PaperAccount,
        current_prices: &HashMap<String, f64>,
    ) {
        let mut to_fill = Vec::new();

        for order in &self.pending_orders {
            if let Some(&current_price) = current_prices.get(&order.symbol) {
                let should_fill = match order.order_type {
                    OrderType::Market => true,
                    OrderType::Limit => {
                        if let Some(limit_price) = order.price {
                            match order.side {
                                Side::Long => current_price <= limit_price,
                                Side::Short => current_price >= limit_price,
                            }
                        } else {
                            false
                        }
                    }
                    OrderType::StopLoss => {
                        if let Some(stop_price) = order.price {
                            match order.side {
                                Side::Long => current_price >= stop_price,
                                Side::Short => current_price <= stop_price,
                            }
                        } else {
                            false
                        }
                    }
                    OrderType::TakeProfit => {
                        if let Some(take_profit_price) = order.price {
                            match order.side {
                                Side::Long => current_price >= take_profit_price,
                                Side::Short => current_price <= take_profit_price,
                            }
                        } else {
                            false
                        }
                    }
                };

                if should_fill {
                    to_fill.push(order.id);
                }
            }
        }

        // Исполняем подходящие ордера
        for order_id in to_fill {
            if let Some(pos) = self
                .pending_orders
                .iter()
                .position(|o| o.id == order_id)
            {
                let mut order = self.pending_orders.remove(pos).unwrap();
                let current_price = current_prices[&order.symbol];

                let result = match order.side {
                    Side::Long => account.open_long(&order.symbol, order.quantity, current_price),
                    Side::Short => {
                        account.open_short(&order.symbol, order.quantity, current_price)
                    }
                };

                match result {
                    Ok(msg) => {
                        order.status = OrderStatus::Filled;
                        order.filled_at = Some(PaperAccount::get_timestamp());
                        println!("Ордер #{} исполнен: {}", order.id, msg);
                    }
                    Err(err) => {
                        order.status = OrderStatus::Rejected;
                        println!("Ордер #{} отклонён: {}", order.id, err);
                    }
                }

                self.filled_orders.push(order);
            }
        }
    }

    fn cancel_order(&mut self, order_id: u64) -> Result<(), String> {
        if let Some(pos) = self
            .pending_orders
            .iter()
            .position(|o| o.id == order_id)
        {
            let mut order = self.pending_orders.remove(pos).unwrap();
            order.status = OrderStatus::Cancelled;
            self.filled_orders.push(order);
            Ok(())
        } else {
            Err(format!("Ордер {} не найден среди ожидающих", order_id))
        }
    }

    fn get_pending_count(&self) -> usize {
        self.pending_orders.len()
    }
}

fn main() {
    let mut account = PaperAccount::new(10000.0, 0.001);
    let mut order_book = OrderBook::new();

    println!("=== Paper Trading с биржевым стаканом ===\n");

    // Размещаем несколько ордеров
    let order1 = order_book.place_order("BTC", Side::Long, OrderType::Limit, 0.5, Some(42000.0));
    let order2 = order_book.place_order("ETH", Side::Short, OrderType::Market, 10.0, None);
    let order3 = order_book.place_order("BTC", Side::Long, OrderType::Limit, 0.3, Some(41500.0));

    println!("Размещено {} ордеров\n", order_book.get_pending_count());

    // Симулируем обновление цен
    let mut prices = HashMap::new();
    prices.insert("BTC".to_string(), 42500.0);
    prices.insert("ETH".to_string(), 2500.0);

    println!("Обработка ордеров при BTC: $42,500, ETH: $2,500");
    order_book.process_orders(&mut account, &prices);
    println!("Ожидающих ордеров: {}\n", order_book.get_pending_count());

    // Цена падает
    prices.insert("BTC".to_string(), 41800.0);
    println!("Обработка ордеров при BTC: $41,800");
    order_book.process_orders(&mut account, &prices);
    println!("Ожидающих ордеров: {}\n", order_book.get_pending_count());

    // Отменяем оставшийся ордер
    if let Err(e) = order_book.cancel_order(order3) {
        println!("Не удалось отменить: {}", e);
    } else {
        println!("Ордер #{} отменён", order3);
    }

    println!("\n{}", account.get_stats());
}
```

## Управление рисками в Paper Trading

```rust
#[derive(Debug)]
struct RiskManager {
    max_position_size: f64,     // Максимальный % счёта на позицию
    max_total_exposure: f64,    // Максимальный % счёта во всех позициях
    max_drawdown: f64,          // Максимальная % просадка от пика
    daily_loss_limit: f64,      // Максимальная дневная просадка %
    peak_balance: f64,
    daily_start_balance: f64,
}

impl RiskManager {
    fn new(
        max_position_size: f64,
        max_total_exposure: f64,
        max_drawdown: f64,
        daily_loss_limit: f64,
        initial_balance: f64,
    ) -> Self {
        Self {
            max_position_size,
            max_total_exposure,
            max_drawdown,
            daily_loss_limit,
            peak_balance: initial_balance,
            daily_start_balance: initial_balance,
        }
    }

    fn update_peak(&mut self, current_balance: f64) {
        if current_balance > self.peak_balance {
            self.peak_balance = current_balance;
        }
    }

    fn reset_daily(&mut self, current_balance: f64) {
        self.daily_start_balance = current_balance;
    }

    fn check_position_size(
        &self,
        quantity: f64,
        price: f64,
        account_balance: f64,
    ) -> Result<(), String> {
        let position_value = quantity * price;
        let position_pct = (position_value / account_balance) * 100.0;

        if position_pct > self.max_position_size {
            return Err(format!(
                "Размер позиции {:.1}% превышает лимит {:.1}%",
                position_pct, self.max_position_size
            ));
        }

        Ok(())
    }

    fn check_total_exposure(
        &self,
        account: &PaperAccount,
        current_prices: &HashMap<String, f64>,
    ) -> Result<(), String> {
        let total_exposure: f64 = account
            .positions
            .iter()
            .map(|(symbol, position)| {
                if let Some(&price) = current_prices.get(symbol) {
                    position.quantity * price
                } else {
                    0.0
                }
            })
            .sum();

        let exposure_pct = (total_exposure / account.balance) * 100.0;

        if exposure_pct > self.max_total_exposure {
            return Err(format!(
                "Общая экспозиция {:.1}% превышает лимит {:.1}%",
                exposure_pct, self.max_total_exposure
            ));
        }

        Ok(())
    }

    fn check_drawdown(&self, current_balance: f64) -> Result<(), String> {
        let drawdown = ((self.peak_balance - current_balance) / self.peak_balance) * 100.0;

        if drawdown > self.max_drawdown {
            return Err(format!(
                "Просадка {:.1}% превышает лимит {:.1}%",
                drawdown, self.max_drawdown
            ));
        }

        Ok(())
    }

    fn check_daily_loss(&self, current_balance: f64) -> Result<(), String> {
        let daily_loss =
            ((self.daily_start_balance - current_balance) / self.daily_start_balance) * 100.0;

        if daily_loss > self.daily_loss_limit {
            return Err(format!(
                "Дневная просадка {:.1}% превышает лимит {:.1}%",
                daily_loss, self.daily_loss_limit
            ));
        }

        Ok(())
    }

    fn validate_trade(
        &mut self,
        account: &PaperAccount,
        quantity: f64,
        price: f64,
        current_prices: &HashMap<String, f64>,
    ) -> Result<(), String> {
        self.update_peak(account.balance);

        self.check_position_size(quantity, price, account.balance)?;
        self.check_total_exposure(account, current_prices)?;
        self.check_drawdown(account.balance)?;
        self.check_daily_loss(account.balance)?;

        Ok(())
    }
}

fn main() {
    let mut account = PaperAccount::new(10000.0, 0.001);
    let mut risk_manager = RiskManager::new(
        10.0, // Максимум 10% на позицию
        50.0, // Максимум 50% общей экспозиции
        20.0, // Максимум 20% просадки
        5.0,  // Максимум 5% дневной просадки
        10000.0,
    );

    let mut prices = HashMap::new();
    prices.insert("BTC".to_string(), 42000.0);

    println!("=== Paper Trading с управлением рисками ===\n");

    // Пытаемся открыть позицию
    let quantity = 0.5;
    let price = 42000.0;

    match risk_manager.validate_trade(&account, quantity, price, &prices) {
        Ok(_) => {
            println!("Проверка рисков пройдена!");
            if let Ok(msg) = account.open_long("BTC", quantity, price) {
                println!("{}", msg);
            }
        }
        Err(e) => println!("Проверка рисков не пройдена: {}", e),
    }

    // Пытаемся открыть слишком большую позицию
    let large_quantity = 5.0; // Будет ~200% счёта
    match risk_manager.validate_trade(&account, large_quantity, price, &prices) {
        Ok(_) => println!("Большая позиция одобрена (не должно произойти)"),
        Err(e) => println!("\nБольшая позиция отклонена: {}", e),
    }

    println!("\n{}", account.get_stats());
}
```

## Что мы изучили

| Концепция | Описание |
|-----------|----------|
| **Paper Trading** | Торговля без риска с виртуальными деньгами |
| **Управление позициями** | Открытие и закрытие длинных/коротких позиций |
| **Типы ордеров** | Рыночные, лимитные, стоп-лосс, тейк-профит ордера |
| **Биржевой стакан** | Управление ожидающими и исполненными ордерами |
| **Расчёт PnL** | Реализованная и нереализованная прибыль/убыток |
| **Управление рисками** | Размер позиций, лимиты экспозиции, контроль просадки |
| **Комиссии** | Транзакционные издержки в виртуальной торговле |
| **Торговая статистика** | Win rate, ROI, отслеживание всех сделок |

## Домашнее задание

1. **Расширенный Paper Trading счёт**: Дополните базовый счёт для виртуальной торговли:
   - Поддержка нескольких валют (USD, EUR, BTC)
   - Маржинальная торговля с кредитным плечом (2x, 5x, 10x)
   - Ликвидация при падении маржи ниже поддерживающего уровня
   - Расчёт процентов на заёмные средства
   - Детальный торговый журнал с заметками

2. **Продвинутые типы ордеров**: Реализуйте дополнительные типы ордеров:
   - OCO (One-Cancels-Other): Два ордера, где исполнение одного отменяет другой
   - Трейлинг стоп-лосс: Стоп, двигающийся вместе с прибыльной ценой
   - Айсберг ордера: Большой ордер разбитый на маленькие видимые части
   - Time-in-force: GTC (Good-Till-Cancel), IOC (Immediate-Or-Cancel), FOK (Fill-Or-Kill)

3. **Ребалансировка портфеля**: Создайте систему которая:
   - Поддерживает целевые процентные доли (например, 60% BTC, 30% ETH, 10% стейблкоины)
   - Автоматически ребалансирует когда распределение отклоняется более чем на 5%
   - Минимизирует транзакционные издержки при ребалансировке
   - Поддерживает ребалансировку по расписанию (ежедневно, еженедельно) или по порогу

4. **Аналитика производительности**: Постройте комплексный модуль аналитики:
   - Расчёт коэффициента Шарпа (доходность с учётом риска)
   - Максимальная просадка и время восстановления
   - Отслеживание серий побед/поражений
   - Средняя прибыль на сделку против среднего убытка
   - Profit factor (валовая прибыль / валовой убыток)
   - Экспорт истории торговли в CSV для внешнего анализа
   - Генерация данных для графика кривой капитала

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md)
