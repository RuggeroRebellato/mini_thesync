# MiniThesync: A Minimal Async Runtime in Rust

MiniThesync is a lightweight implementation of an asynchronous runtime in Rust. This project was created as part of a job-related research and testing initiative to gain a deeper understanding of async runtimes like Tokio.

## Project Background

This project was developed in response to a specific job demand that required a thorough understanding of asynchronous runtime mechanisms in Rust. The goal was to explore the inner workings of async runtimes by building a simplified version from scratch. This hands-on approach provided valuable insights into the complexities and design considerations of production-ready runtimes like Tokio.

## Disclaimer

While this project was motivated by professional research, it is not intended for production use. MiniThesync is a simplified implementation that lacks many features and optimizations found in production-ready runtimes. It serves primarily as a learning tool and a demonstration of core async runtime concepts.

## Features

- Basic task spawning and execution
- Work-stealing scheduler
- Support for both I/O-like and CPU-bound tasks

## Benchmark Results

We compared MiniThesync with Tokio, a production-ready async runtime, for both I/O-like and CPU-bound tasks. Here are the results:

### I/O-like tasks (10,000 tasks with 100µs sleep):

- MiniThesync: 98.581245ms
- Tokio: 7.033464ms
- MiniThesync is 14.02x slower than Tokio

### CPU-bound tasks (10,000 tasks with a simple counting loop):

- MiniThesync: 1.94485ms
- Tokio: 3.627551ms
- MiniThesync is 1.87x faster than Tokio

These results show that while MiniThesync is slower for I/O-like tasks, it performs better for CPU-bound tasks in this specific benchmark scenario.

## Usage

To use MiniThesync in your project, add it to your `Cargo.toml`:

```toml
[dependencies]
mini_thesync = { git = "https://github.com/RuggeroRebellato/mini_thesync.git" }
```

Then, you can use it in your code:

```rust
use mini_thesync::Runtime;
use std::time::Duration;

fn main() {
    let runtime = Runtime::new(num_cpus::get());

    runtime.spawn(async {
        println!("Hello from MiniThesync!");
        mini_thesync::sleep(Duration::from_millis(100)).await;
        println!("Goodbye from MiniThesync!");
    });

    runtime.run();
}
```

## Running the Benchmarks

To run the benchmarks comparing MiniThesync with Tokio:

1. Clone the repository:

   ```
   git clone https://github.com/RuggeroRebellato/mini_thesync.git
   cd mini_thesync
   ```

2. Run the benchmark:
   ```
   cargo bench
   ```

## Contributing

This project is primarily for educational purposes, but if you have ideas for improvements or want to experiment with the implementation, feel free to open an issue or submit a pull request.

## License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Acknowledgments

- The Rust Async Book for providing a great resource on async concepts in Rust.
- The Tokio project for inspiration and being a reference for async runtime implementation.

## Disclaimer

This project is not affiliated with or endorsed by the Tokio project or its maintainers. It is an independent, educational implementation created for learning purposes.
