use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, Condvar};
use std::collections::BinaryHeap;
use std::thread;
use std::cmp::{Ord, PartialOrd, Ordering, Eq};

struct Entry {
	pub instant: Instant,
	pub callback: Box<FnMut() + Send>
}

impl PartialOrd for Entry {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Eq for Entry {}

impl PartialEq for Entry {
	fn eq(&self, other: &Self) -> bool {
		self.instant == other.instant
	}
}

impl Ord for Entry {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.instant.cmp(&other.instant) {
			Ordering::Greater => Ordering::Less,
			Ordering::Less => Ordering::Greater,
			Ordering::Equal => Ordering::Equal
		}
	}
}

struct SharedData {
	pub cond_var: Condvar,
	pub callbacks: Mutex<BinaryHeap<Entry>>
}

pub struct Scheduler {
	data: Arc<SharedData>
}

impl Scheduler {
	pub fn new() -> Scheduler {
		let shared_data = Arc::new(SharedData {
			cond_var: Condvar::new(),
			callbacks: Mutex::new(BinaryHeap::new())
		});
		{
			let shared_data = shared_data.clone();
			thread::spawn(move || {
				let mut callbacks = shared_data.callbacks.lock().unwrap();
				loop {
					let entry = callbacks.pop();
					match entry {
						Some(mut entry) => {
							let now = Instant::now();
							if entry.instant > now {
								let wait_duration = entry.instant - now;
								callbacks.push(entry);
								callbacks = shared_data.cond_var
										.wait_timeout(callbacks, wait_duration).unwrap().0;
							} else {
								(entry.callback)()
							}
						}
						None => {
							callbacks = shared_data.cond_var.wait(callbacks).unwrap();
						}
					}
				}
			});
		}

		Scheduler {
			data: shared_data
		}
	}

	pub fn after_instant<F>(&self, instant: Instant, func: F)
		where F: FnOnce() + Send + 'static {
		let mut func = Some(func);
		self.data.callbacks.lock().unwrap().push(Entry {
			instant,
			callback: Box::new(move || {
				if let Some(func) = func.take() {
					(func)()
				}
			}),
		});
		self.data.cond_var.notify_all();
	}

	pub fn after_duration<F>(&self, duration: Duration, func: F)
		where F: FnOnce() + Send + 'static {
		self.after_instant(Instant::now() + duration, func)
	}
}

#[test]
fn test() {
	use std::sync::atomic::{AtomicBool, Ordering};

	let atomic = Arc::new(AtomicBool::new(false));
	let scheduler = Scheduler::new();
	{
		let atomic = atomic.clone();
		scheduler.after_instant(Instant::now() + Duration::from_millis(10), move || {
			atomic.store(true, Ordering::Relaxed);
		});
	}
	thread::sleep(Duration::from_millis(100));
	assert_eq!(atomic.load(Ordering::Relaxed), true);
}
