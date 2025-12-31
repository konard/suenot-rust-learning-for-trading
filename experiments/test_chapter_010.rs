// Test code from Chapter 010: Characters and Strings

fn main() {
    println!("=== Testing char examples ===");
    let currency_sign: char = '$';
    let btc_symbol: char = '₿';
    let up_arrow: char = '↑';
    let down_arrow: char = '↓';
    let check_mark: char = '✓';

    println!("Bitcoin {} {}", btc_symbol, up_arrow);
    println!("Trade executed {}", check_mark);

    println!("\n=== Testing emoji chars ===");
    let rocket: char = '🚀';
    let money: char = '💰';
    let chart: char = '📈';

    println!("To the moon! {} {} {}", rocket, money, chart);

    println!("\n=== Testing &str ===");
    let ticker: &str = "BTC/USDT";
    let exchange: &str = "Binance";
    let status: &str = "Order filled";

    println!("Trading {} on {}", ticker, exchange);
    println!("Status: {}", status);

    println!("\n=== Testing String ===");
    let mut message = String::from("Price: ");
    message.push_str("42000");
    message.push_str(" USDT");
    println!("{}", message);
    message.push('!');
    println!("{}", message);

    println!("\n=== Testing String creation ===");
    let s1 = String::from("BTC");
    let s2 = "ETH".to_string();
    let s3 = String::new();
    let s4: String = format!("{}/{}", "BTC", "USDT");

    println!("s1: {}", s1);
    println!("s2: {}", s2);
    println!("s3: '{}'", s3);
    println!("s4: {}", s4);

    println!("\n=== Testing concatenation ===");
    let base = "BTC";
    let quote = "USDT";
    let pair = format!("{}/{}", base, quote);
    println!("Pair: {}", pair);

    let mut s = String::from("Order #");
    s = s + "12345";
    println!("{}", s);

    let mut log = String::from("[INFO] ");
    log.push_str("Trade executed at ");
    log.push_str("42000 USDT");
    println!("{}", log);

    println!("\n=== Testing String methods ===");
    let ticker = String::from("  BTC/USDT  ");
    println!("Trimmed: '{}'", ticker.trim());
    println!("Length: {}", ticker.len());
    println!("Contains BTC: {}", ticker.contains("BTC"));
    println!("Starts with BTC: {}", ticker.trim().starts_with("BTC"));
    println!("Ends with USDT: {}", ticker.trim().ends_with("USDT"));
    let new_pair = ticker.trim().replace("USDT", "EUR");
    println!("New pair: {}", new_pair);
    println!("Upper: {}", ticker.to_uppercase());
    println!("Lower: {}", ticker.to_lowercase());
    let empty = String::new();
    println!("Is empty: {}", empty.is_empty());

    println!("\n=== Testing split ===");
    let pair = "BTC/USDT";
    let parts: Vec<&str> = pair.split('/').collect();
    println!("Base: {}", parts[0]);
    println!("Quote: {}", parts[1]);
    for part in pair.split('/') {
        println!("Part: {}", part);
    }

    println!("\n=== Testing chars ===");
    let ticker = "BTC";
    for c in ticker.chars() {
        println!("Char: {}", c);
    }
    let first = ticker.chars().next();
    println!("First: {:?}", first);
    let second = ticker.chars().nth(1);
    println!("Second: {:?}", second);

    println!("\n=== Testing trading pair parsing ===");
    let pair = "ETH/USDT";
    let parts: Vec<&str> = pair.split('/').collect();
    if parts.len() == 2 {
        let base = parts[0];
        let quote = parts[1];
        println!("Trading pair: {}", pair);
        println!("Base currency: {}", base);
        println!("Quote currency: {}", quote);
        let stablecoins = ["USDT", "USDC", "BUSD", "DAI"];
        let is_stable_pair = stablecoins.contains(&quote);
        println!("Stable pair: {}", is_stable_pair);
    } else {
        println!("Invalid pair format!");
    }

    println!("\n=== Testing message formatting ===");
    let symbol = "BTC/USDT";
    let side = "BUY";
    let price = 42000.50;
    let quantity = 0.5;
    let order_id = 123456789;
    let log_message = format!(
        "[TRADE] {} {} {:.8} @ {:.2} | Order #{}",
        side, symbol, quantity, price, order_id
    );
    println!("{}", log_message);
    let user_message = format!(
        "Bought {} {} at ${:.2} each. Total: ${:.2}",
        quantity, symbol.split('/').next().unwrap_or(""),
        price, price * quantity
    );
    println!("{}", user_message);
    let api_payload = format!(
        r#"{{"symbol":"{}","side":"{}","price":{},"quantity":{}}}"#,
        symbol, side, price, quantity
    );
    println!("API: {}", api_payload);

    println!("\n=== Testing ticker validation ===");
    let tickers = ["BTC", "eth", "XRP123", "DOGE", ""];
    for ticker in tickers {
        let is_valid = validate_ticker(ticker);
        println!("{:10} -> valid: {}", format!("'{}'", ticker), is_valid);
    }

    println!("\n=== Testing &str <-> String conversion ===");
    let s: &str = "BTC";
    let string1: String = s.to_string();
    let string2: String = String::from(s);
    let string3: String = s.to_owned();

    let string = String::from("ETH");
    let slice: &str = &string;
    let slice2: &str = string.as_str();

    println!("String: {}", string);
    println!("Slice: {}", slice);
    println!("Slice2: {}", slice2);

    println!("\n=== All tests passed! ===");
}

fn validate_ticker(ticker: &str) -> bool {
    if ticker.is_empty() {
        return false;
    }
    if ticker.len() < 2 || ticker.len() > 10 {
        return false;
    }
    ticker.chars().all(|c| c.is_ascii_uppercase())
}
