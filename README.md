# Task Scheduler

A library to easily schedule an FnOnce to run sometime in the future.
It is guaranteed that the task will not run _before_ the given time, but
due to delays may run slightly after.

The timer uses a single thread to schedule and run all tasks. A long-running
task will delay other tasks from running.



## Example

```rust
extern crate task_scheduler;

use task_scheduler::Scheduler;

let scheduler = Scheduler::new();

//pick some time in the future
let time = Instant::now() + Duration::from_secs(5);
scheduler.after_instant(time, ||{
    println!("do something here")
});

scheduler.after_duration(Duration::from_millis(100), ||{
    println!("do something else")
});

```