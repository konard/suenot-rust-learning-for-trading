// Test code examples from Chapter 175

mod simple_thread_pool {
    use std::sync::{Arc, Mutex, mpsc};
    use std::thread;

    type Job = Box<dyn FnOnce() + Send + 'static>;

    pub struct ThreadPool {
        workers: Vec<Worker>,
        sender: Option<mpsc::Sender<Job>>,
    }

    struct Worker {
        #[allow(dead_code)]
        id: usize,
        thread: Option<thread::JoinHandle<()>>,
    }

    impl ThreadPool {
        pub fn new(size: usize) -> ThreadPool {
            let (sender, receiver) = mpsc::channel();
            let receiver = Arc::new(Mutex::new(receiver));

            let mut workers = Vec::with_capacity(size);

            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }

            ThreadPool {
                workers,
                sender: Some(sender),
            }
        }

        pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static,
        {
            let job = Box::new(f);
            self.sender.as_ref().unwrap().send(job).unwrap();
        }
    }

    impl Drop for ThreadPool {
        fn drop(&mut self) {
            drop(self.sender.take());

            for worker in &mut self.workers {
                if let Some(thread) = worker.thread.take() {
                    thread.join().unwrap();
                }
            }
        }
    }

    impl Worker {
        fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
            let thread = thread::spawn(move || loop {
                let job = receiver.lock().unwrap().recv();

                match job {
                    Ok(job) => {
                        println!("Worker {} received a job", id);
                        job();
                    }
                    Err(_) => {
                        println!("Worker {} shutting down", id);
                        break;
                    }
                }
            });

            Worker {
                id,
                thread: Some(thread),
            }
        }
    }
}

mod rayon_example {
    use rayon::prelude::*;

    #[derive(Debug, Clone)]
    pub struct StockAnalysis {
        pub symbol: String,
        pub price: f64,
        pub sma_20: f64,
        pub sma_50: f64,
        pub signal: Signal,
    }

    #[derive(Debug, Clone)]
    pub enum Signal {
        Buy,
        Sell,
        Hold,
    }

    pub fn analyze_stock(symbol: &str, prices: &[f64]) -> StockAnalysis {
        let sma_20 = if prices.len() >= 20 {
            prices.iter().rev().take(20).sum::<f64>() / 20.0
        } else {
            prices.iter().sum::<f64>() / prices.len() as f64
        };

        let sma_50 = if prices.len() >= 50 {
            prices.iter().rev().take(50).sum::<f64>() / 50.0
        } else {
            prices.iter().sum::<f64>() / prices.len() as f64
        };

        let current_price = *prices.last().unwrap();

        let signal = if sma_20 > sma_50 && current_price > sma_20 {
            Signal::Buy
        } else if sma_20 < sma_50 && current_price < sma_20 {
            Signal::Sell
        } else {
            Signal::Hold
        };

        StockAnalysis {
            symbol: symbol.to_string(),
            price: current_price,
            sma_20,
            sma_50,
            signal,
        }
    }

    pub fn run_analysis() {
        let stocks: Vec<(String, Vec<f64>)> = (0..100)
            .map(|i| {
                let symbol = format!("STOCK_{:03}", i);
                let prices: Vec<f64> = (0..100)
                    .map(|j| 100.0 + (i as f64 * 0.1) + (j as f64 * 0.01))
                    .collect();
                (symbol, prices)
            })
            .collect();

        let results: Vec<StockAnalysis> = stocks
            .par_iter()
            .map(|(symbol, prices)| analyze_stock(symbol, prices))
            .collect();

        let buy_signals: Vec<_> = results
            .iter()
            .filter(|a| matches!(a.signal, Signal::Buy))
            .collect();

        println!("Found {} buy signals:", buy_signals.len());
        for analysis in buy_signals.iter().take(5) {
            println!(
                "  {} @ ${:.2} (SMA20: {:.2}, SMA50: {:.2})",
                analysis.symbol, analysis.price, analysis.sma_20, analysis.sma_50
            );
        }
    }
}

mod order_executor {
    use std::sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}};
    use rayon::prelude::*;

    #[derive(Debug, Clone)]
    pub struct Order {
        pub id: u64,
        pub symbol: String,
        pub side: OrderSide,
        pub quantity: f64,
        pub price: f64,
    }

    #[derive(Debug, Clone)]
    pub enum OrderSide {
        Buy,
        Sell,
    }

    #[derive(Debug, Clone)]
    pub struct ExecutionResult {
        pub order_id: u64,
        pub status: ExecutionStatus,
        pub filled_price: Option<f64>,
    }

    #[derive(Debug, Clone)]
    pub enum ExecutionStatus {
        Filled,
        PartiallyFilled,
        Rejected(String),
    }

    pub struct OrderExecutor {
        #[allow(dead_code)]
        order_counter: AtomicU64,
        execution_log: Arc<Mutex<Vec<ExecutionResult>>>,
    }

    impl OrderExecutor {
        pub fn new() -> Self {
            OrderExecutor {
                order_counter: AtomicU64::new(0),
                execution_log: Arc::new(Mutex::new(Vec::new())),
            }
        }

        pub fn execute_order(&self, order: &Order) -> ExecutionResult {
            std::thread::sleep(std::time::Duration::from_millis(10));

            let result = if order.quantity > 1000.0 {
                ExecutionResult {
                    order_id: order.id,
                    status: ExecutionStatus::Rejected(
                        "Order size too large".to_string()
                    ),
                    filled_price: None,
                }
            } else {
                let slippage = match order.side {
                    OrderSide::Buy => 1.001,
                    OrderSide::Sell => 0.999,
                };

                ExecutionResult {
                    order_id: order.id,
                    status: ExecutionStatus::Filled,
                    filled_price: Some(order.price * slippage),
                }
            };

            self.execution_log.lock().unwrap().push(result.clone());
            result
        }

        pub fn process_batch(&self, orders: Vec<Order>) -> Vec<ExecutionResult> {
            orders
                .par_iter()
                .map(|order| self.execute_order(order))
                .collect()
        }

        pub fn get_stats(&self) -> (usize, usize, usize) {
            let log = self.execution_log.lock().unwrap();
            let filled = log.iter()
                .filter(|r| matches!(r.status, ExecutionStatus::Filled))
                .count();
            let partial = log.iter()
                .filter(|r| matches!(r.status, ExecutionStatus::PartiallyFilled))
                .count();
            let rejected = log.iter()
                .filter(|r| matches!(r.status, ExecutionStatus::Rejected(_)))
                .count();
            (filled, partial, rejected)
        }
    }

    pub fn run_executor() {
        let executor = OrderExecutor::new();

        let orders: Vec<Order> = (0..20)
            .map(|i| Order {
                id: i,
                symbol: if i % 2 == 0 { "BTC" } else { "ETH" }.to_string(),
                side: if i % 3 == 0 { OrderSide::Sell } else { OrderSide::Buy },
                quantity: 10.0 + (i as f64 * 50.0),
                price: 42000.0 + (i as f64 * 10.0),
            })
            .collect();

        println!("Processing {} orders...", orders.len());

        let start = std::time::Instant::now();
        let results = executor.process_batch(orders);
        let elapsed = start.elapsed();

        println!("Processed in {:?}", elapsed);

        let (filled, partial, rejected) = executor.get_stats();
        println!("\nResults:");
        println!("  Filled: {}", filled);
        println!("  Partial: {}", partial);
        println!("  Rejected: {}", rejected);

        println!("\nExecution examples:");
        for result in results.iter().take(5) {
            match &result.status {
                ExecutionStatus::Filled => {
                    println!(
                        "  Order {}: filled @ ${:.2}",
                        result.order_id,
                        result.filled_price.unwrap()
                    );
                }
                ExecutionStatus::Rejected(reason) => {
                    println!("  Order {}: rejected - {}", result.order_id, reason);
                }
                ExecutionStatus::PartiallyFilled => {
                    println!("  Order {}: partially filled", result.order_id);
                }
            }
        }
    }
}

fn main() {
    println!("=== Testing Simple Thread Pool ===\n");
    {
        let pool = simple_thread_pool::ThreadPool::new(4);

        for i in 0..10 {
            pool.execute(move || {
                let symbol = format!("STOCK_{}", i);
                println!("Analyzing {}", symbol);
                std::thread::sleep(std::time::Duration::from_millis(50));
                println!("{}: analysis complete", symbol);
            });
        }
    }

    println!("\n=== Testing rayon Stock Analysis ===\n");
    rayon_example::run_analysis();

    println!("\n=== Testing Order Executor ===\n");
    order_executor::run_executor();

    println!("\nAll tests passed!");
}
