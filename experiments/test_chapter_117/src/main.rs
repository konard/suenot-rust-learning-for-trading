use std::time::Duration;
use tokio::time::sleep;

// Test 1: Basic async with Result
async fn async_fetch_price() -> Result<f64, String> {
    Ok(42000.0)
}

// Test 2: tokio::join! example
async fn fetch_btc_price() -> Result<f64, String> {
    Ok(42000.0)
}

async fn fetch_eth_price() -> Result<f64, String> {
    Err("ETH API unavailable".to_string())
}

async fn fetch_sol_price() -> Result<f64, String> {
    Ok(95.0)
}

// Test 3: tokio::spawn with nested Result
async fn risky_trade() -> Result<f64, String> {
    Err("Trade failed: insufficient balance".to_string())
}

// Test 4: try_join! example
async fn fetch_btc_ok() -> Result<f64, String> {
    Ok(42000.0)
}

async fn fetch_eth_err() -> Result<f64, String> {
    Err("ETH API down".to_string())
}

// Test 5: select! example
async fn fetch_from_binance() -> Result<f64, String> {
    sleep(Duration::from_millis(100)).await;
    Ok(42000.0)
}

async fn fetch_from_kraken() -> Result<f64, String> {
    sleep(Duration::from_millis(150)).await;
    Ok(42050.0)
}

// Test 6: PriceMonitor example
struct PriceMonitor {
    symbol: String,
    retry_count: u32,
    max_retries: u32,
}

impl PriceMonitor {
    fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            retry_count: 0,
            max_retries: 3,
        }
    }

    async fn fetch_price(&self) -> Result<f64, String> {
        if rand::random::<f64>() < 0.3 {
            Err(format!("{} API error", self.symbol))
        } else {
            Ok(42000.0 * rand::random::<f64>())
        }
    }

    async fn run_once(&mut self) -> bool {
        match self.fetch_price().await {
            Ok(price) => {
                println!("[{}] Price: ${:.2}", self.symbol, price);
                self.retry_count = 0;
                true
            }
            Err(e) => {
                self.retry_count += 1;
                println!("[{}] Error (attempt {}): {}", self.symbol, self.retry_count, e);
                self.retry_count < self.max_retries
            }
        }
    }
}

// Test 7: ExchangeClient with retry
struct ExchangeClient {
    name: String,
    max_retries: u32,
    retry_delay: Duration,
}

#[derive(Debug)]
enum ExchangeError {
    ConnectionFailed(String),
    RateLimited,
    InvalidResponse(String),
    Timeout,
}

impl std::fmt::Display for ExchangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            Self::RateLimited => write!(f, "Rate limited"),
            Self::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            Self::Timeout => write!(f, "Request timed out"),
        }
    }
}

impl std::error::Error for ExchangeError {}

impl ExchangeClient {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
        }
    }

    async fn fetch_price(&self, _symbol: &str) -> Result<f64, ExchangeError> {
        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            match self.try_fetch_price().await {
                Ok(price) => return Ok(price),
                Err(e) => {
                    println!("[{}] Attempt {}/{} failed: {}", self.name, attempt, self.max_retries, e);
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        let delay = self.retry_delay * (2_u32.pow(attempt - 1));
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    async fn try_fetch_price(&self) -> Result<f64, ExchangeError> {
        let random: f64 = rand::random();

        if random < 0.2 {
            Err(ExchangeError::ConnectionFailed("Network error".to_string()))
        } else if random < 0.3 {
            Err(ExchangeError::RateLimited)
        } else if random < 0.35 {
            Err(ExchangeError::Timeout)
        } else {
            Ok(42000.0 + (random * 1000.0))
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Test 1: Basic async with Result ===");
    let result = async_fetch_price().await;
    println!("Price: {:?}", result);

    println!("\n=== Test 2: tokio::join! ===");
    let (btc, eth, sol) = tokio::join!(
        fetch_btc_price(),
        fetch_eth_price(),
        fetch_sol_price()
    );
    println!("BTC: {:?}", btc);
    println!("ETH: {:?}", eth);
    println!("SOL: {:?}", sol);

    println!("\n=== Test 3: tokio::spawn with nested Result ===");
    let handle = tokio::spawn(async {
        risky_trade().await
    });
    match handle.await {
        Ok(Ok(profit)) => println!("Profit: ${:.2}", profit),
        Ok(Err(e)) => println!("Trade error: {}", e),
        Err(e) => println!("Task panic: {:?}", e),
    }

    println!("\n=== Test 4: try_join! ===");
    let result = tokio::try_join!(
        fetch_btc_ok(),
        fetch_eth_err(),
        fetch_sol_price()
    );
    match result {
        Ok((btc, eth, sol)) => {
            println!("All prices: BTC={}, ETH={}, SOL={}", btc, eth, sol);
        }
        Err(e) => {
            println!("Failed to fetch prices: {}", e);
        }
    }

    println!("\n=== Test 5: select! ===");
    tokio::select! {
        result = fetch_from_binance() => {
            match result {
                Ok(price) => println!("Binance: ${}", price),
                Err(e) => println!("Binance error: {}", e),
            }
        }
        result = fetch_from_kraken() => {
            match result {
                Ok(price) => println!("Kraken: ${}", price),
                Err(e) => println!("Kraken error: {}", e),
            }
        }
    }

    println!("\n=== Test 6: PriceMonitor ===");
    let mut monitor = PriceMonitor::new("BTC");
    for _ in 0..3 {
        if !monitor.run_once().await {
            break;
        }
    }

    println!("\n=== Test 7: ExchangeClient with retry ===");
    let client = ExchangeClient::new("Binance");
    match client.fetch_price("BTCUSDT").await {
        Ok(price) => println!("BTC price: ${:.2}", price),
        Err(e) => println!("Failed after all retries: {}", e),
    }

    println!("\n=== All tests completed ===");
}
