# День 343: Docker: контейнеризация бота

## Аналогия из трейдинга

Представь, что ты работаешь в крупной инвестиционной компании. У тебя есть торговый бот, который отлично работает на твоём компьютере. Но как перенести его на сервер биржи или в облако?

**Проблема:** На твоём компьютере установлены определённые версии библиотек, Rust, и конфигурации. На сервере — всё может быть иначе. Это как если бы трейдер привык к одному терминалу, а его посадили за другой — кнопки в других местах, горячие клавиши не работают.

**Решение Docker:** Контейнер — это как персональный торговый терминал, который ты можешь взять с собой куда угодно. Внутри контейнера — всё настроено именно так, как тебе нужно: версии библиотек, конфигурация, переменные окружения. Запустил на любом сервере — работает одинаково.

| Аналогия | Docker концепция |
|----------|-----------------|
| Торговый терминал в чемодане | Docker контейнер |
| Инструкция по настройке терминала | Dockerfile |
| Склад готовых терминалов | Docker Registry (Docker Hub) |
| Образ терминала на диске | Docker Image |
| Запущенный терминал | Docker Container |

## Что такое Docker?

Docker — это платформа для разработки, доставки и запуска приложений в изолированных контейнерах. Контейнер содержит всё необходимое для работы приложения: код, runtime, библиотеки, переменные окружения.

### Ключевые понятия

```
┌─────────────────────────────────────────────────────────────┐
│                         Docker Host                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │  Container 1    │  │  Container 2    │  │  Container 3 │ │
│  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌────────┐  │ │
│  │  │Trading Bot│  │  │  │ Database  │  │  │  │ Redis  │  │ │
│  │  └───────────┘  │  │  └───────────┘  │  │  └────────┘  │ │
│  │  Rust 1.75     │  │  │  PostgreSQL   │  │  │  Redis 7   │ │
│  │  Alpine Linux  │  │  │  Alpine Linux │  │  │  Alpine    │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
│                                                              │
│                    Docker Engine                             │
└─────────────────────────────────────────────────────────────┘
                              │
                    Host Operating System
```

## Установка Docker

### Linux (Ubuntu/Debian)

```bash
# Установка Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Добавляем пользователя в группу docker
sudo usermod -aG docker $USER

# Проверка
docker --version
```

### macOS и Windows

Скачайте Docker Desktop с [docker.com](https://www.docker.com/products/docker-desktop/).

## Создание Dockerfile для торгового бота

Начнём с простого торгового бота и создадим для него Dockerfile:

### Структура проекта

```
trading-bot/
├── Cargo.toml
├── src/
│   └── main.rs
├── Dockerfile
└── .dockerignore
```

### Cargo.toml

```toml
[package]
name = "trading-bot"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### src/main.rs — Простой торговый бот

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tracing::{info, warn, error, Level};
use tracing_subscriber::FmtSubscriber;

/// Рыночные данные
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MarketData {
    symbol: String,
    bid: f64,
    ask: f64,
    last_price: f64,
    volume: f64,
    timestamp: u64,
}

/// Ордер
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    id: String,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    status: OrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
}

/// Конфигурация бота из переменных окружения
#[derive(Debug)]
struct BotConfig {
    /// Символы для торговли
    symbols: Vec<String>,
    /// Максимальный размер позиции
    max_position_size: f64,
    /// Порог для входа (спред в процентах)
    entry_threshold: f64,
    /// API ключ (в реальности - из секретов)
    api_key: String,
    /// Режим работы: live или paper
    mode: String,
}

impl BotConfig {
    /// Загрузка конфигурации из переменных окружения
    fn from_env() -> Self {
        let symbols = env::var("BOT_SYMBOLS")
            .unwrap_or_else(|_| "BTCUSDT,ETHUSDT".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let max_position_size = env::var("BOT_MAX_POSITION")
            .unwrap_or_else(|_| "1.0".to_string())
            .parse()
            .unwrap_or(1.0);

        let entry_threshold = env::var("BOT_ENTRY_THRESHOLD")
            .unwrap_or_else(|_| "0.1".to_string())
            .parse()
            .unwrap_or(0.1);

        let api_key = env::var("BOT_API_KEY")
            .unwrap_or_else(|_| "demo_key".to_string());

        let mode = env::var("BOT_MODE")
            .unwrap_or_else(|_| "paper".to_string());

        BotConfig {
            symbols,
            max_position_size,
            entry_threshold,
            api_key,
            mode,
        }
    }
}

/// Торговый движок
struct TradingEngine {
    config: BotConfig,
    positions: HashMap<String, f64>,
    orders: Vec<Order>,
    order_counter: u64,
}

impl TradingEngine {
    fn new(config: BotConfig) -> Self {
        TradingEngine {
            config,
            positions: HashMap::new(),
            orders: Vec::new(),
            order_counter: 0,
        }
    }

    /// Обработка рыночных данных
    fn process_market_data(&mut self, data: &MarketData) {
        let spread = (data.ask - data.bid) / data.bid * 100.0;

        info!(
            symbol = %data.symbol,
            bid = data.bid,
            ask = data.ask,
            spread = format!("{:.4}%", spread),
            "Получены рыночные данные"
        );

        // Простая стратегия: покупаем при узком спреде
        if spread < self.config.entry_threshold {
            let current_position = self.positions.get(&data.symbol).unwrap_or(&0.0);

            if *current_position < self.config.max_position_size {
                self.place_order(&data.symbol, OrderSide::Buy, data.ask, 0.1);
            }
        }
    }

    /// Размещение ордера
    fn place_order(&mut self, symbol: &str, side: OrderSide, price: f64, quantity: f64) {
        self.order_counter += 1;
        let order_id = format!("ORD-{:06}", self.order_counter);

        let order = Order {
            id: order_id.clone(),
            symbol: symbol.to_string(),
            side: side.clone(),
            price,
            quantity,
            status: OrderStatus::Pending,
        };

        info!(
            order_id = %order_id,
            symbol = %symbol,
            side = ?side,
            price = price,
            quantity = quantity,
            "Размещён ордер"
        );

        // В режиме paper - сразу исполняем
        if self.config.mode == "paper" {
            self.execute_order(&order);
        }

        self.orders.push(order);
    }

    /// Исполнение ордера
    fn execute_order(&mut self, order: &Order) {
        let position = self.positions.entry(order.symbol.clone()).or_insert(0.0);

        match order.side {
            OrderSide::Buy => *position += order.quantity,
            OrderSide::Sell => *position -= order.quantity,
        }

        info!(
            order_id = %order.id,
            new_position = *position,
            "Ордер исполнен"
        );
    }

    /// Получение текущих позиций
    fn get_positions(&self) -> &HashMap<String, f64> {
        &self.positions
    }

    /// Вывод статистики
    fn print_stats(&self) {
        info!("=== Статистика торгового бота ===");
        info!("Режим: {}", self.config.mode);
        info!("Символы: {:?}", self.config.symbols);
        info!("Всего ордеров: {}", self.orders.len());
        info!("Текущие позиции:");
        for (symbol, size) in &self.positions {
            info!("  {}: {:.4}", symbol, size);
        }
    }
}

/// Симуляция получения рыночных данных
fn simulate_market_data(symbol: &str, base_price: f64) -> MarketData {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Добавляем небольшую волатильность
    let noise = ((timestamp % 100) as f64 - 50.0) / 1000.0;
    let price = base_price * (1.0 + noise);

    MarketData {
        symbol: symbol.to_string(),
        bid: price * 0.9999,
        ask: price * 1.0001,
        last_price: price,
        volume: 1000.0 + (timestamp % 500) as f64,
        timestamp,
    }
}

#[tokio::main]
async fn main() {
    // Инициализация логирования
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .json()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Ошибка установки подписчика логов");

    info!("Запуск торгового бота...");

    // Загружаем конфигурацию
    let config = BotConfig::from_env();
    info!(config = ?config, "Конфигурация загружена");

    // Создаём торговый движок
    let mut engine = TradingEngine::new(config);

    // Базовые цены для симуляции
    let base_prices: HashMap<&str, f64> = [
        ("BTCUSDT", 50000.0),
        ("ETHUSDT", 3000.0),
    ]
    .into_iter()
    .collect();

    // Основной цикл
    let mut iteration = 0;
    loop {
        iteration += 1;
        info!(iteration = iteration, "--- Итерация ---");

        // Обрабатываем каждый символ
        for symbol in &engine.config.symbols.clone() {
            if let Some(&base_price) = base_prices.get(symbol.as_str()) {
                let market_data = simulate_market_data(symbol, base_price);
                engine.process_market_data(&market_data);
            } else {
                warn!(symbol = %symbol, "Неизвестный символ, пропускаем");
            }
        }

        // Выводим статистику каждые 5 итераций
        if iteration % 5 == 0 {
            engine.print_stats();
        }

        // Ждём перед следующей итерацией
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Для демонстрации — выходим после 20 итераций
        if iteration >= 20 {
            info!("Достигнут лимит итераций, завершаем работу");
            break;
        }
    }

    info!("Торговый бот завершил работу");
    engine.print_stats();
}
```

### Dockerfile — Многоэтапная сборка

```dockerfile
# ==========================================
# Этап 1: Сборка (Builder)
# ==========================================
FROM rust:1.75-alpine AS builder

# Устанавливаем необходимые пакеты для сборки
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

# Создаём рабочую директорию
WORKDIR /app

# Копируем файлы зависимостей для кеширования слоёв
COPY Cargo.toml Cargo.lock* ./

# Создаём фиктивный main.rs для сборки зависимостей
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Собираем только зависимости (это будет закешировано)
RUN cargo build --release && rm -rf src

# Копируем исходный код
COPY src ./src

# Пересобираем с актуальным кодом
RUN touch src/main.rs && cargo build --release

# ==========================================
# Этап 2: Runtime (минимальный образ)
# ==========================================
FROM alpine:3.19 AS runtime

# Устанавливаем минимальные зависимости
RUN apk add --no-cache ca-certificates tzdata

# Создаём непривилегированного пользователя
RUN addgroup -S trading && adduser -S bot -G trading

# Рабочая директория
WORKDIR /app

# Копируем бинарник из builder
COPY --from=builder /app/target/release/trading-bot /app/trading-bot

# Меняем владельца
RUN chown -R bot:trading /app

# Переключаемся на непривилегированного пользователя
USER bot

# Переменные окружения по умолчанию
ENV BOT_MODE=paper \
    BOT_SYMBOLS=BTCUSDT,ETHUSDT \
    BOT_MAX_POSITION=1.0 \
    BOT_ENTRY_THRESHOLD=0.1 \
    RUST_LOG=info

# Точка входа
ENTRYPOINT ["/app/trading-bot"]
```

### .dockerignore — Исключаем ненужные файлы

```
# Результаты сборки
target/
Cargo.lock

# Git
.git/
.gitignore

# IDE
.idea/
.vscode/
*.swp
*.swo

# Документация и тесты (для продакшн образа)
docs/
tests/
benches/
examples/

# Docker файлы
Dockerfile*
docker-compose*.yml
.dockerignore

# Логи и временные файлы
*.log
*.tmp
.env.local
```

## Сборка и запуск контейнера

```bash
# Сборка образа
docker build -t trading-bot:latest .

# Просмотр созданного образа
docker images | grep trading-bot

# Запуск контейнера
docker run --name my-bot \
    -e BOT_MODE=paper \
    -e BOT_SYMBOLS=BTCUSDT,ETHUSDT \
    -e BOT_MAX_POSITION=0.5 \
    trading-bot:latest

# Запуск в фоновом режиме
docker run -d --name my-bot-daemon \
    -e BOT_MODE=paper \
    trading-bot:latest

# Просмотр логов
docker logs -f my-bot-daemon

# Остановка контейнера
docker stop my-bot-daemon

# Удаление контейнера
docker rm my-bot-daemon
```

## Оптимизация Docker образа

### Сравнение размеров образов

```rust
// Пример скрипта для анализа размера образа
use std::process::Command;

fn get_image_size(image_name: &str) -> String {
    let output = Command::new("docker")
        .args(["images", image_name, "--format", "{{.Size}}"])
        .output()
        .expect("Failed to execute docker command");

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn main() {
    let images = vec![
        ("rust:latest", "Базовый Rust образ"),
        ("rust:slim", "Slim Rust образ"),
        ("rust:alpine", "Alpine Rust образ"),
        ("trading-bot:latest", "Наш оптимизированный бот"),
    ];

    println!("=== Сравнение размеров Docker образов ===\n");

    for (image, description) in images {
        let size = get_image_size(image);
        println!("{}: {} ({})", description, size, image);
    }
}
```

### Типичные размеры:

| Образ | Размер | Описание |
|-------|--------|----------|
| `rust:latest` | ~1.4 GB | Полный Rust toolchain |
| `rust:slim` | ~800 MB | Без лишних утилит |
| `rust:alpine` | ~600 MB | На базе Alpine |
| Наш бот (multi-stage) | ~15-30 MB | Только бинарник |

## Работа с секретами

### Безопасная передача API ключей

```bash
# Плохо: ключ виден в истории команд и docker inspect
docker run -e BOT_API_KEY=super_secret_key trading-bot:latest

# Лучше: из файла
echo "super_secret_key" > api_key.txt
docker run --env-file .env trading-bot:latest

# Ещё лучше: Docker secrets (в Swarm/Kubernetes)
echo "super_secret_key" | docker secret create bot_api_key -
```

### .env файл для разработки

```env
# .env
BOT_MODE=paper
BOT_SYMBOLS=BTCUSDT,ETHUSDT
BOT_MAX_POSITION=1.0
BOT_ENTRY_THRESHOLD=0.1
BOT_API_KEY=dev_key_12345
```

## Health Checks в Docker

```dockerfile
# Добавляем в Dockerfile
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1
```

### Добавляем health endpoint в бота

```rust
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;

/// Простой HTTP сервер для health checks
async fn health_server() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await.expect("Не удалось запустить health server");

    info!("Health server запущен на {}", addr);

    loop {
        if let Ok((mut socket, _)) = listener.accept().await {
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"healthy\"}";
            let _ = socket.write_all(response.as_bytes()).await;
        }
    }
}

// В main() добавляем:
// tokio::spawn(health_server());
```

## Логирование для контейнеров

### Структурированные логи в JSON

```rust
use tracing_subscriber::fmt::format::JsonFields;

fn setup_logging() {
    // JSON логи для Docker/Kubernetes
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .with_current_span(false)
        .with_span_list(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Ошибка установки логгера");
}
```

Пример вывода:

```json
{"timestamp":"2024-01-15T10:30:00.123Z","level":"INFO","message":"Получены рыночные данные","symbol":"BTCUSDT","bid":50000.5,"ask":50001.0}
{"timestamp":"2024-01-15T10:30:00.125Z","level":"INFO","message":"Размещён ордер","order_id":"ORD-000001","symbol":"BTCUSDT","side":"Buy","price":50001.0}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Docker Image** | Неизменяемый шаблон для создания контейнеров |
| **Docker Container** | Запущенный экземпляр образа |
| **Dockerfile** | Инструкции для сборки образа |
| **Multi-stage build** | Многоэтапная сборка для минимизации размера |
| **.dockerignore** | Исключение файлов из контекста сборки |
| **Environment variables** | Конфигурация через переменные окружения |
| **Health checks** | Проверка работоспособности контейнера |
| **JSON logging** | Структурированные логи для контейнерных сред |

## Практические задания

1. **Оптимизация образа**: Измени Dockerfile, чтобы использовать `scratch` вместо `alpine` как базовый образ. Какой размер получился? Какие проблемы возникли?

2. **Конфигурация через файл**: Добавь поддержку загрузки конфигурации из TOML файла, смонтированного в контейнер:
   ```bash
   docker run -v $(pwd)/config.toml:/app/config.toml trading-bot:latest
   ```

3. **Graceful shutdown**: Реализуй корректное завершение бота при получении сигнала `SIGTERM` (Docker отправляет его при `docker stop`):
   ```rust
   tokio::select! {
       _ = trading_loop() => {}
       _ = tokio::signal::ctrl_c() => {
           info!("Получен сигнал завершения");
       }
   }
   ```

4. **Метрики для мониторинга**: Добавь endpoint `/metrics` с метриками в формате Prometheus:
   - Количество обработанных ордеров
   - Текущие позиции
   - Время работы бота

## Домашнее задание

1. **Полноценный Dockerfile с кешированием**: Создай Dockerfile, который:
   - Использует cargo-chef для оптимального кеширования зависимостей
   - Включает health check
   - Имеет отдельные стадии для dev и prod
   - Финальный образ весит менее 20 МБ

2. **Мульти-архитектурная сборка**: Настрой сборку образа для ARM64 и AMD64:
   ```bash
   docker buildx build --platform linux/amd64,linux/arm64 -t trading-bot:multi .
   ```

3. **Локальный registry**: Разверни локальный Docker Registry и опубликуй туда свой образ:
   ```bash
   docker run -d -p 5000:5000 registry:2
   docker tag trading-bot:latest localhost:5000/trading-bot:latest
   docker push localhost:5000/trading-bot:latest
   ```

4. **CI/CD интеграция**: Напиши GitHub Actions workflow, который:
   - Собирает Docker образ при каждом push
   - Запускает тесты внутри контейнера
   - Публикует образ в GitHub Container Registry
   - Сканирует образ на уязвимости с помощью Trivy

5. **Продвинутое логирование**: Реализуй систему логирования, которая:
   - Выводит JSON логи в stdout
   - Ротирует логи по размеру (в случае записи в файл)
   - Включает request_id для трассировки
   - Маскирует чувствительные данные (API ключи)

## Навигация

[← Предыдущий день](../342-*/ru.md) | [Следующий день →](../344-*/ru.md)
