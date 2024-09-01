use futures::future::join_all;
use mini_thesync::Runtime as MiniRuntime;
use std::time::{Duration, Instant};

const TASK_COUNT: usize = 10_000;
const TASK_DURATION_MICROS: u64 = 100;

fn bench_mini_thesync() -> Duration {
    let runtime = MiniRuntime::new(num_cpus::get());
    let start = Instant::now();

    for _ in 0..TASK_COUNT {
        runtime.spawn(async {
            std::thread::sleep(Duration::from_micros(TASK_DURATION_MICROS));
        });
    }

    runtime.run();
    start.elapsed()
}

async fn bench_tokio() -> Duration {
    let start = Instant::now();

    let tasks: Vec<_> = (0..TASK_COUNT)
        .map(|_| {
            tokio::spawn(async {
                tokio::time::sleep(Duration::from_micros(TASK_DURATION_MICROS)).await;
            })
        })
        .collect();

    join_all(tasks).await;
    start.elapsed()
}

fn bench_cpu_bound_mini_thesync() -> Duration {
    let runtime = MiniRuntime::new(num_cpus::get());
    let start = Instant::now();

    for _ in 0..TASK_COUNT {
        runtime.spawn(async {
            // Simulate CPU-bound work
            for (ref mut x, _) in (0..10_000).enumerate() {
                *x += 1;
            }
        });
    }

    runtime.run();
    start.elapsed()
}

async fn bench_cpu_bound_tokio() -> Duration {
    let start = Instant::now();

    let tasks: Vec<_> = (0..TASK_COUNT)
        .map(|_| {
            tokio::spawn(async {
                // Simulate CPU-bound work
                for (ref mut x, _) in (0..10_000).enumerate() {
                    *x += 1;
                }
            })
        })
        .collect();

    join_all(tasks).await;
    start.elapsed()
}

fn main() {
    println!("Running MiniAsync I/O-like benchmark...");
    let mini_io_time = bench_mini_thesync();
    println!("MiniAsync I/O-like time: {:?}", mini_io_time);

    println!("\nRunning Tokio I/O-like benchmark...");
    let tokio_io_time = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(bench_tokio());
    println!("Tokio I/O-like time: {:?}", tokio_io_time);

    println!("\nRunning MiniAsync CPU-bound benchmark...");
    let mini_cpu_time = bench_cpu_bound_mini_thesync();
    println!("MiniAsync CPU-bound time: {:?}", mini_cpu_time);

    println!("\nRunning Tokio CPU-bound benchmark...");
    let tokio_cpu_time = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(bench_cpu_bound_tokio());
    println!("Tokio CPU-bound time: {:?}", tokio_cpu_time);

    println!("\nComparison for I/O-like tasks:");
    println!("MiniAsync: {:?}", mini_io_time);
    println!("Tokio: {:?}", tokio_io_time);
    println!(
        "MiniAsync is {:.2}x {} than Tokio for I/O-like tasks",
        if mini_io_time > tokio_io_time {
            mini_io_time.as_secs_f64() / tokio_io_time.as_secs_f64()
        } else {
            tokio_io_time.as_secs_f64() / mini_io_time.as_secs_f64()
        },
        if mini_io_time > tokio_io_time {
            "slower"
        } else {
            "faster"
        }
    );

    println!("\nComparison for CPU-bound tasks:");
    println!("MiniAsync: {:?}", mini_cpu_time);
    println!("Tokio: {:?}", tokio_cpu_time);
    println!(
        "MiniAsync is {:.2}x {} than Tokio for CPU-bound tasks",
        if mini_cpu_time > tokio_cpu_time {
            mini_cpu_time.as_secs_f64() / tokio_cpu_time.as_secs_f64()
        } else {
            tokio_cpu_time.as_secs_f64() / mini_cpu_time.as_secs_f64()
        },
        if mini_cpu_time > tokio_cpu_time {
            "slower"
        } else {
            "faster"
        }
    );
}
