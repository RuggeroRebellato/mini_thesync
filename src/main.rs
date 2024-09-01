use mini_thesync::Runtime;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    // Create a new runtime with a number of threads equal to the number of CPU cores
    let runtime = Arc::new(Runtime::new(num_cpus::get()));

    // Spawn 10 tasks
    for i in 0..10 {
        runtime.spawn(async move {
            // Simulate some asynchronous work
            mini_thesync::sleep(Duration::from_millis(100 * (i as u64 + 1))).await;
            println!("Task {} completed", i);
        });
    }

    // Spawn a task that spawns another task
    let runtime_clone = runtime.clone();
    runtime.spawn(async move {
        println!("Parent task started");
        mini_thesync::sleep(Duration::from_millis(500)).await;

        runtime_clone.spawn(async {
            mini_thesync::sleep(Duration::from_millis(200)).await;
            println!("Child task completed");
        });

        println!("Parent task completed");
    });

    // Run the runtime
    println!("Starting runtime");
    runtime.run();
    println!("All tasks completed");
}
