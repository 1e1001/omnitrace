//! pretty thread naming

use std::thread;

pub fn spawn<T: Send + 'static, F: FnOnce() -> T + Send + 'static>(
	name: impl Into<String>,
	f: F,
) -> thread::JoinHandle<T> {
	thread::Builder::new()
		.name(name.into())
		.spawn(f)
		.expect("Failed to start thread")
}

pub fn spawn_scoped<'scope, 'env, T: Send + 'scope, F: FnOnce() -> T + Send + 'scope>(
	name: impl Into<String>,
	scope: &'scope thread::Scope<'scope, 'env>,
	f: F,
) -> thread::ScopedJoinHandle<'scope, T> {
	thread::Builder::new()
		.name(name.into())
		.spawn_scoped(scope, f)
		.expect("Failed to start thread")
}
