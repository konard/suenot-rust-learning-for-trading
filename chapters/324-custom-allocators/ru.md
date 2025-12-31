# День 324: Custom Allocators: свои аллокаторы

## Аналогия из трейдинга

Представь, что ты управляешь высокочастотной торговой фирмой с тысячами сделок в секунду. Стандартный банковский сервис обрабатывает каждую транзакцию индивидуально — с проверками, логированием, очередями. Это надёжно, но медленно.

Теперь представь, что ты создаёшь **собственный внутренний расчётный центр**:
- **Пулы капитала** — заранее зарезервированные деньги для быстрых расчётов
- **Арена для сделок** — область памяти, где все сделки сессии живут вместе
- **Стек быстрых операций** — мгновенное выделение и освобождение для временных расчётов

**Custom allocator** в Rust — это твой собственный расчётный центр для памяти:
- **Стандартный аллокатор** = банк с общими правилами
- **Пул аллокатор** = зарезервированный капитал для быстрых операций
- **Арена аллокатор** = торговая сессия, где все объекты освобождаются разом
- **Bump аллокатор** = стек для сверхбыстрых временных вычислений

В высокочастотной торговле каждая микросекунда на счету. Стандартный аллокатор может вызывать непредсказуемые паузы из-за фрагментации памяти или системных вызовов. Собственные аллокаторы дают контроль и предсказуемость.

## Зачем нужны custom allocators?

| Проблема | Решение | Пример в трейдинге |
|----------|---------|-------------------|
| **Фрагментация** | Пул фиксированных блоков | Ордера одного размера |
| **Паузы на аллокацию** | Предвыделенная арена | Все сделки сессии |
| **Медленное освобождение** | Bump allocator | Временные расчёты индикаторов |
| **Отладка утечек** | Трассирующий аллокатор | Мониторинг памяти в production |
| **Детерминизм** | Статический буфер | Критические пути исполнения |

## Базовая структура аллокатора в Rust

Rust предоставляет трейт `GlobalAlloc` для создания собственных аллокаторов:

```rust
use std::alloc::{GlobalAlloc, Layout};

/// Трейт для глобального аллокатора
unsafe trait GlobalAlloc {
    /// Выделяет память
    unsafe fn alloc(&self, layout: Layout) -> *mut u8;

    /// Освобождает память
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);

    /// Перевыделяет память (опционально)
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // Реализация по умолчанию
    }

    /// Выделяет инициализированную нулями память (опционально)
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // Реализация по умолчанию
    }
}
```

### Минимальный пример: обёртка над System

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Аллокатор со счётчиками для торговой системы
struct TradingAllocator {
    allocations: AtomicUsize,
    deallocations: AtomicUsize,
    bytes_allocated: AtomicUsize,
    peak_usage: AtomicUsize,
}

impl TradingAllocator {
    const fn new() -> Self {
        TradingAllocator {
            allocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            bytes_allocated: AtomicUsize::new(0),
            peak_usage: AtomicUsize::new(0),
        }
    }

    fn stats(&self) -> (usize, usize, usize, usize) {
        (
            self.allocations.load(Ordering::Relaxed),
            self.deallocations.load(Ordering::Relaxed),
            self.bytes_allocated.load(Ordering::Relaxed),
            self.peak_usage.load(Ordering::Relaxed),
        )
    }
}

unsafe impl GlobalAlloc for TradingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            self.allocations.fetch_add(1, Ordering::Relaxed);
            let new_total = self.bytes_allocated.fetch_add(layout.size(), Ordering::Relaxed)
                + layout.size();

            // Обновляем пиковое использование
            let mut current_peak = self.peak_usage.load(Ordering::Relaxed);
            while new_total > current_peak {
                match self.peak_usage.compare_exchange_weak(
                    current_peak,
                    new_total,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(p) => current_peak = p,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocations.fetch_add(1, Ordering::Relaxed);
        self.bytes_allocated.fetch_sub(layout.size(), Ordering::Relaxed);
        System.dealloc(ptr, layout);
    }
}

#[global_allocator]
static ALLOCATOR: TradingAllocator = TradingAllocator::new();

fn main() {
    // Симуляция торговой активности
    let mut orders: Vec<String> = Vec::new();

    for i in 0..1000 {
        orders.push(format!("Order-{}-BTCUSDT", i));
    }

    let (allocs, deallocs, bytes, peak) = ALLOCATOR.stats();
    println!("=== Статистика аллокатора ===");
    println!("Аллокаций: {}", allocs);
    println!("Деаллокаций: {}", deallocs);
    println!("Текущее использование: {} KB", bytes / 1024);
    println!("Пиковое использование: {} KB", peak / 1024);
}
```

## Pool Allocator: пул фиксированных блоков

Идеален для объектов одинакового размера — например, ордеров:

```rust
use std::alloc::{GlobalAlloc, Layout};
use std::cell::UnsafeCell;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// Размер блока в пуле (достаточно для типичного ордера)
const BLOCK_SIZE: usize = 128;
/// Количество блоков в пуле
const POOL_SIZE: usize = 10000;

/// Блок в пуле
#[repr(C, align(16))]
struct PoolBlock {
    data: [u8; BLOCK_SIZE],
}

/// Узел свободного списка
struct FreeNode {
    next: AtomicPtr<FreeNode>,
}

/// Пул аллокатор для ордеров
struct OrderPoolAllocator {
    // Статический буфер
    pool: UnsafeCell<[PoolBlock; POOL_SIZE]>,
    // Голова свободного списка
    free_list: AtomicPtr<FreeNode>,
    // Статистика
    allocations: AtomicUsize,
    pool_hits: AtomicUsize,
    pool_misses: AtomicUsize,
}

unsafe impl Sync for OrderPoolAllocator {}

impl OrderPoolAllocator {
    const fn new() -> Self {
        OrderPoolAllocator {
            pool: UnsafeCell::new([PoolBlock { data: [0; BLOCK_SIZE] }; POOL_SIZE]),
            free_list: AtomicPtr::new(std::ptr::null_mut()),
            allocations: AtomicUsize::new(0),
            pool_hits: AtomicUsize::new(0),
            pool_misses: AtomicUsize::new(0),
        }
    }

    /// Инициализация свободного списка
    unsafe fn init(&self) {
        let pool = &mut *self.pool.get();

        for i in 0..POOL_SIZE - 1 {
            let current = &mut pool[i] as *mut PoolBlock as *mut FreeNode;
            let next = &mut pool[i + 1] as *mut PoolBlock as *mut FreeNode;
            (*current).next = AtomicPtr::new(next);
        }

        let last = &mut pool[POOL_SIZE - 1] as *mut PoolBlock as *mut FreeNode;
        (*last).next = AtomicPtr::new(std::ptr::null_mut());

        self.free_list.store(&mut pool[0] as *mut PoolBlock as *mut FreeNode, Ordering::Release);
    }

    /// Проверяем, принадлежит ли указатель пулу
    fn is_from_pool(&self, ptr: *mut u8) -> bool {
        let pool_start = self.pool.get() as *mut u8;
        let pool_end = unsafe { pool_start.add(POOL_SIZE * std::mem::size_of::<PoolBlock>()) };
        ptr >= pool_start && ptr < pool_end
    }

    fn stats(&self) -> (usize, usize, usize) {
        (
            self.allocations.load(Ordering::Relaxed),
            self.pool_hits.load(Ordering::Relaxed),
            self.pool_misses.load(Ordering::Relaxed),
        )
    }
}

unsafe impl GlobalAlloc for OrderPoolAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocations.fetch_add(1, Ordering::Relaxed);

        // Если размер подходит для пула
        if layout.size() <= BLOCK_SIZE && layout.align() <= 16 {
            // Пытаемся взять блок из свободного списка (lock-free)
            loop {
                let head = self.free_list.load(Ordering::Acquire);
                if head.is_null() {
                    break; // Пул пуст
                }

                let next = (*head).next.load(Ordering::Relaxed);

                if self.free_list.compare_exchange_weak(
                    head,
                    next,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() {
                    self.pool_hits.fetch_add(1, Ordering::Relaxed);
                    return head as *mut u8;
                }
            }
        }

        // Fallback на системный аллокатор
        self.pool_misses.fetch_add(1, Ordering::Relaxed);
        std::alloc::System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Если блок из пула — возвращаем в свободный список
        if self.is_from_pool(ptr) {
            let node = ptr as *mut FreeNode;
            loop {
                let head = self.free_list.load(Ordering::Acquire);
                (*node).next = AtomicPtr::new(head);

                if self.free_list.compare_exchange_weak(
                    head,
                    node,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() {
                    return;
                }
            }
        }

        // Иначе освобождаем через системный аллокатор
        std::alloc::System.dealloc(ptr, layout);
    }
}

// Пример использования
fn main() {
    #[repr(C)]
    struct Order {
        id: u64,
        symbol: [u8; 16],
        price: f64,
        quantity: f64,
        side: u8,
        _padding: [u8; 7],
    }

    println!("Размер Order: {} bytes", std::mem::size_of::<Order>());

    // В реальном коде:
    // static ALLOCATOR: OrderPoolAllocator = OrderPoolAllocator::new();
    // unsafe { ALLOCATOR.init(); }

    // Симуляция высокочастотной торговли
    let mut orders = Vec::with_capacity(1000);

    for i in 0..1000 {
        orders.push(Box::new(Order {
            id: i,
            symbol: *b"BTCUSDT\0\0\0\0\0\0\0\0\0",
            price: 50000.0 + i as f64,
            quantity: 0.01,
            side: if i % 2 == 0 { 0 } else { 1 },
            _padding: [0; 7],
        }));
    }

    println!("Создано {} ордеров", orders.len());
}
```

## Arena Allocator: память для сессии

Арена выделяет память последовательно и освобождает всё разом в конце сессии:

```rust
use std::alloc::Layout;
use std::cell::UnsafeCell;
use std::ptr::NonNull;

/// Арена для торговой сессии
struct TradingArena {
    /// Буфер памяти
    buffer: UnsafeCell<Vec<u8>>,
    /// Текущая позиция
    offset: UnsafeCell<usize>,
    /// Ёмкость
    capacity: usize,
}

impl TradingArena {
    fn new(capacity: usize) -> Self {
        TradingArena {
            buffer: UnsafeCell::new(vec![0u8; capacity]),
            offset: UnsafeCell::new(0),
            capacity,
        }
    }

    /// Выделяет память в арене
    fn alloc<T>(&self) -> Option<NonNull<T>> {
        let layout = Layout::new::<T>();
        self.alloc_layout(layout).map(|ptr| ptr.cast())
    }

    fn alloc_layout(&self, layout: Layout) -> Option<NonNull<u8>> {
        unsafe {
            let offset = &mut *self.offset.get();
            let buffer = &mut *self.buffer.get();

            // Выравнивание
            let align_offset = (*offset).wrapping_neg() & (layout.align() - 1);
            let new_offset = *offset + align_offset + layout.size();

            if new_offset > self.capacity {
                return None; // Арена заполнена
            }

            let ptr = buffer.as_mut_ptr().add(*offset + align_offset);
            *offset = new_offset;

            NonNull::new(ptr)
        }
    }

    /// Выделяет память для слайса
    fn alloc_slice<T>(&self, count: usize) -> Option<&mut [T]> {
        let layout = Layout::array::<T>(count).ok()?;
        let ptr = self.alloc_layout(layout)?;

        unsafe {
            Some(std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut T, count))
        }
    }

    /// Сбрасывает арену для переиспользования
    fn reset(&self) {
        unsafe {
            *self.offset.get() = 0;
        }
    }

    /// Использованная память
    fn used(&self) -> usize {
        unsafe { *self.offset.get() }
    }

    /// Оставшаяся память
    fn remaining(&self) -> usize {
        self.capacity - self.used()
    }
}

// Пример: торговая сессия с ареной
#[derive(Debug, Clone, Copy)]
struct Trade {
    timestamp: u64,
    price: f64,
    quantity: f64,
    is_buy: bool,
}

#[derive(Debug, Clone, Copy)]
struct PriceLevel {
    price: f64,
    volume: f64,
}

fn main() {
    // Создаём арену на 1 MB для торговой сессии
    let arena = TradingArena::new(1024 * 1024);

    println!("=== Торговая сессия ===");
    println!("Ёмкость арены: {} KB", arena.capacity / 1024);

    // Выделяем массив для сделок
    let trades: &mut [Trade] = arena.alloc_slice(10000).expect("Не хватило памяти");

    // Заполняем данными
    for (i, trade) in trades.iter_mut().enumerate() {
        *trade = Trade {
            timestamp: 1700000000000 + i as u64,
            price: 50000.0 + (i as f64 * 0.1),
            quantity: 0.01 + (i as f64 * 0.001),
            is_buy: i % 2 == 0,
        };
    }

    println!("Записано {} сделок", trades.len());
    println!("Использовано: {} KB", arena.used() / 1024);
    println!("Осталось: {} KB", arena.remaining() / 1024);

    // Выделяем уровни цен для стакана
    let bid_levels: &mut [PriceLevel] = arena.alloc_slice(100).expect("Не хватило памяти");
    let ask_levels: &mut [PriceLevel] = arena.alloc_slice(100).expect("Не хватило памяти");

    // Инициализируем стакан
    for (i, level) in bid_levels.iter_mut().enumerate() {
        *level = PriceLevel {
            price: 50000.0 - i as f64,
            volume: 1.0 + i as f64 * 0.1,
        };
    }

    for (i, level) in ask_levels.iter_mut().enumerate() {
        *level = PriceLevel {
            price: 50000.0 + i as f64,
            volume: 1.0 + i as f64 * 0.1,
        };
    }

    println!("Стакан: {} bid / {} ask уровней", bid_levels.len(), ask_levels.len());
    println!("Итого использовано: {} KB", arena.used() / 1024);

    // Вычисляем средневзвешенную цену
    let vwap: f64 = trades.iter()
        .map(|t| t.price * t.quantity)
        .sum::<f64>()
        / trades.iter().map(|t| t.quantity).sum::<f64>();

    println!("VWAP: {:.2}", vwap);

    // Конец сессии — сбрасываем арену
    arena.reset();
    println!("\n=== Сессия завершена ===");
    println!("Арена сброшена, использовано: {} bytes", arena.used());
}
```

## Bump Allocator: сверхбыстрое выделение

Bump allocator — простейший и самый быстрый аллокатор:

```rust
use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::alloc::Layout;

/// Маркер времени жизни для безопасного заимствования
struct BumpMarker<'a> {
    _phantom: PhantomData<&'a ()>,
}

/// Сверхбыстрый bump allocator
struct BumpAllocator<'a> {
    start: *mut u8,
    end: *mut u8,
    ptr: Cell<*mut u8>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> BumpAllocator<'a> {
    /// Создаёт аллокатор из буфера
    fn from_slice(buffer: &'a mut [u8]) -> Self {
        let start = buffer.as_mut_ptr();
        let end = unsafe { start.add(buffer.len()) };

        BumpAllocator {
            start,
            end,
            ptr: Cell::new(start),
            _marker: PhantomData,
        }
    }

    /// Выделяет память
    fn alloc<T>(&self, value: T) -> Option<&'a mut T> {
        let layout = Layout::new::<T>();

        let current = self.ptr.get();

        // Выравниваем
        let aligned = (current as usize + layout.align() - 1) & !(layout.align() - 1);
        let new_ptr = aligned + layout.size();

        if new_ptr > self.end as usize {
            return None;
        }

        self.ptr.set(new_ptr as *mut u8);

        unsafe {
            let ptr = aligned as *mut T;
            ptr.write(value);
            Some(&mut *ptr)
        }
    }

    /// Выделяет память для среза
    fn alloc_slice<T: Copy>(&self, values: &[T]) -> Option<&'a mut [T]> {
        let layout = Layout::array::<T>(values.len()).ok()?;

        let current = self.ptr.get();
        let aligned = (current as usize + layout.align() - 1) & !(layout.align() - 1);
        let new_ptr = aligned + layout.size();

        if new_ptr > self.end as usize {
            return None;
        }

        self.ptr.set(new_ptr as *mut u8);

        unsafe {
            let ptr = aligned as *mut T;
            std::ptr::copy_nonoverlapping(values.as_ptr(), ptr, values.len());
            Some(std::slice::from_raw_parts_mut(ptr, values.len()))
        }
    }

    /// Использованная память
    fn used(&self) -> usize {
        self.ptr.get() as usize - self.start as usize
    }

    /// Оставшаяся память
    fn remaining(&self) -> usize {
        self.end as usize - self.ptr.get() as usize
    }

    /// Сброс (освобождает всё сразу)
    fn reset(&self) {
        self.ptr.set(self.start);
    }
}

// Пример: быстрые вычисления индикаторов
fn main() {
    // Статический буфер для временных вычислений
    let mut buffer = [0u8; 64 * 1024]; // 64 KB

    let bump = BumpAllocator::from_slice(&mut buffer);

    println!("=== Быстрый расчёт индикаторов ===");

    // Входные данные — цены
    let prices = [
        50000.0, 50100.0, 50050.0, 50200.0, 50150.0,
        50300.0, 50250.0, 50400.0, 50350.0, 50500.0,
    ];

    // Выделяем память для промежуточных вычислений
    let prices_copy = bump.alloc_slice(&prices).unwrap();
    println!("Копия цен: {:?}", &prices_copy[..5]);

    // Рассчитываем доходности
    let returns: Vec<f64> = prices.windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect();

    let returns_slice = bump.alloc_slice(&returns).unwrap();
    println!("Доходности: {:?}", &returns_slice[..5]);

    // SMA (простая скользящая средняя)
    let period = 5;
    let mut sma_values = Vec::new();

    for i in period..=prices.len() {
        let sum: f64 = prices[i - period..i].iter().sum();
        sma_values.push(sum / period as f64);
    }

    let sma = bump.alloc_slice(&sma_values).unwrap();
    println!("SMA(5): {:?}", sma);

    // Волатильность (стандартное отклонение)
    let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance: f64 = returns.iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>() / (returns.len() - 1) as f64;
    let volatility = variance.sqrt() * (252.0_f64).sqrt(); // Годовая волатильность

    let vol = bump.alloc(volatility).unwrap();
    println!("Годовая волатильность: {:.2}%", vol * 100.0);

    println!("\nИспользовано памяти: {} bytes", bump.used());
    println!("Осталось: {} bytes", bump.remaining());

    // Сброс для следующего расчёта
    bump.reset();
    println!("После сброса: {} bytes", bump.used());
}
```

## Сравнение аллокаторов

| Аллокатор | Скорость alloc | Скорость dealloc | Фрагментация | Использование |
|-----------|----------------|------------------|--------------|---------------|
| **System** | Средняя | Средняя | Да | Общего назначения |
| **Pool** | Быстрая | Быстрая | Нет* | Объекты одного размера |
| **Arena** | Очень быстрая | Мгновенная (bulk) | Нет | Данные сессии |
| **Bump** | Мгновенная | Невозможна | Нет | Временные вычисления |

*при условии одинакового размера блоков

## Практический пример: торговый движок с custom allocators

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

/// Метрики аллокатора для production
struct AllocatorMetrics {
    total_allocations: AtomicU64,
    total_deallocations: AtomicU64,
    current_bytes: AtomicUsize,
    peak_bytes: AtomicUsize,
    alloc_time_ns: AtomicU64,
    dealloc_time_ns: AtomicU64,
}

impl AllocatorMetrics {
    const fn new() -> Self {
        AllocatorMetrics {
            total_allocations: AtomicU64::new(0),
            total_deallocations: AtomicU64::new(0),
            current_bytes: AtomicUsize::new(0),
            peak_bytes: AtomicUsize::new(0),
            alloc_time_ns: AtomicU64::new(0),
            dealloc_time_ns: AtomicU64::new(0),
        }
    }
}

/// Production-ready аллокатор для торгового движка
struct TradingEngineAllocator {
    metrics: AllocatorMetrics,
}

impl TradingEngineAllocator {
    const fn new() -> Self {
        TradingEngineAllocator {
            metrics: AllocatorMetrics::new(),
        }
    }

    fn print_stats(&self) {
        let allocs = self.metrics.total_allocations.load(Ordering::Relaxed);
        let deallocs = self.metrics.total_deallocations.load(Ordering::Relaxed);
        let current = self.metrics.current_bytes.load(Ordering::Relaxed);
        let peak = self.metrics.peak_bytes.load(Ordering::Relaxed);
        let alloc_time = self.metrics.alloc_time_ns.load(Ordering::Relaxed);
        let dealloc_time = self.metrics.dealloc_time_ns.load(Ordering::Relaxed);

        println!("=== Trading Engine Allocator Stats ===");
        println!("Total allocations:   {}", allocs);
        println!("Total deallocations: {}", deallocs);
        println!("Outstanding:         {}", allocs - deallocs);
        println!("Current memory:      {} KB", current / 1024);
        println!("Peak memory:         {} KB", peak / 1024);
        if allocs > 0 {
            println!("Avg alloc time:      {} ns", alloc_time / allocs);
        }
        if deallocs > 0 {
            println!("Avg dealloc time:    {} ns", dealloc_time / deallocs);
        }
    }
}

unsafe impl GlobalAlloc for TradingEngineAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let start = Instant::now();

        let ptr = System.alloc(layout);

        if !ptr.is_null() {
            self.metrics.total_allocations.fetch_add(1, Ordering::Relaxed);

            let new_size = self.metrics.current_bytes.fetch_add(layout.size(), Ordering::Relaxed)
                + layout.size();

            // Обновляем пик
            let mut current_peak = self.metrics.peak_bytes.load(Ordering::Relaxed);
            while new_size > current_peak {
                match self.metrics.peak_bytes.compare_exchange_weak(
                    current_peak,
                    new_size,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(p) => current_peak = p,
                }
            }
        }

        let elapsed = start.elapsed().as_nanos() as u64;
        self.metrics.alloc_time_ns.fetch_add(elapsed, Ordering::Relaxed);

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let start = Instant::now();

        self.metrics.total_deallocations.fetch_add(1, Ordering::Relaxed);
        self.metrics.current_bytes.fetch_sub(layout.size(), Ordering::Relaxed);

        System.dealloc(ptr, layout);

        let elapsed = start.elapsed().as_nanos() as u64;
        self.metrics.dealloc_time_ns.fetch_add(elapsed, Ordering::Relaxed);
    }
}

#[global_allocator]
static ALLOCATOR: TradingEngineAllocator = TradingEngineAllocator::new();

// Симуляция торгового движка
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    side: Side,
}

#[derive(Debug, Clone, Copy)]
enum Side {
    Buy,
    Sell,
}

fn main() {
    println!("Запуск торгового движка...\n");

    let start = Instant::now();

    // Симуляция высокочастотной торговли
    let mut orders: Vec<Order> = Vec::with_capacity(10000);

    for i in 0..10000 {
        orders.push(Order {
            id: i,
            symbol: "BTCUSDT".to_string(),
            price: 50000.0 + (i as f64 * 0.1),
            quantity: 0.01 + (i as f64 * 0.0001),
            side: if i % 2 == 0 { Side::Buy } else { Side::Sell },
        });
    }

    println!("Создано {} ордеров за {:?}", orders.len(), start.elapsed());

    // Обработка ордеров
    let matched: Vec<_> = orders.iter()
        .filter(|o| matches!(o.side, Side::Buy) && o.price > 50500.0)
        .collect();

    println!("Исполнено {} ордеров", matched.len());

    // Очистка
    drop(orders);

    println!();
    ALLOCATOR.print_stats();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **GlobalAlloc** | Трейт для создания глобальных аллокаторов |
| **Pool Allocator** | Пул фиксированных блоков для объектов одного размера |
| **Arena Allocator** | Последовательное выделение с bulk-освобождением |
| **Bump Allocator** | Сверхбыстрое выделение без индивидуального освобождения |
| **Lock-free** | Безблокировочные алгоритмы для многопоточности |
| **Memory metrics** | Сбор статистики для мониторинга production |

## Практические задания

1. **Thread-local arena**: Создай арену для каждого потока торгового движка:
   - Каждый поток имеет свою арену
   - Нет блокировок при выделении
   - Сброс в конце каждой торговой сессии
   - Метрики использования по потокам

2. **Slab allocator**: Реализуй slab аллокатор:
   - Несколько пулов для разных размеров объектов
   - Автоматический выбор подходящего пула
   - Fallback на системный аллокатор для больших объектов
   - Сбор статистики по размерам

3. **Ring buffer allocator**: Создай кольцевой аллокатор для потока данных:
   - Фиксированный размер буфера
   - FIFO семантика — старые данные перезаписываются
   - Оптимизирован для потока tick-данных
   - Нет фрагментации

4. **Leak detector wrapper**: Разработай обёртку-детектор утечек:
   - Оборачивает любой аллокатор
   - Отслеживает все аллокации с callstack
   - Выводит отчёт о незакрытых аллокациях
   - Интеграция с CI/CD

## Домашнее задание

1. **Гибридный аллокатор**: Создай аллокатор для торгового бота:
   - Pool для ордеров (частые, одинаковый размер)
   - Arena для данных сессии
   - System для редких больших объектов
   - Prometheus метрики для каждого типа
   - Документация с бенчмарками

2. **Memory-mapped аллокатор**: Реализуй аллокатор на mmap:
   - Выделяет память через mmap для больших блоков
   - Использует пулы для мелких объектов
   - Поддержка huge pages для производительности
   - Тесты на выравнивание и корректность

3. **Deterministic allocator**: Создай детерминированный аллокатор:
   - Предсказуемое время аллокации (без системных вызовов)
   - Статический пул с максимальным размером
   - Panic при переполнении (fail-fast)
   - Идеально для hot path торгового движка

4. **Allocation profiler**: Разработай профилировщик аллокаций:
   - Записывает все аллокации с временными метками
   - Строит histogram по размерам
   - Определяет hot spots
   - Генерирует flamegraph аллокаций
   - Интеграция с существующим торговым ботом

## Навигация

[← Предыдущий день](../323-zero-copy-avoiding-copies/ru.md) | [Следующий день →](../325-jemalloc-mimalloc/ru.md)
