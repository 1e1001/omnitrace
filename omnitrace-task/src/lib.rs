#![feature(allocator_api)]

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};

pub mod render;

#[derive(Debug, Clone)]
pub enum TaskEvent {
	Begin(Box<str>, Option<u64>),
	End,
	Progress(u64),
}

#[derive(Debug, Clone)]
pub struct Section {
	pub name: Box<str>,
	pub progress: Option<(u64, u64)>,
	pub start: SystemTime,
}
#[derive(Debug, Clone)]
pub struct FinishedSection {
	pub name: Box<str>,
	pub progress: Option<(u64, u64)>,
	pub duration: Duration,
}
pub trait AnySection {
	fn name(&self) -> &str;
	fn progress(&self) -> Option<(u64, u64)>;
	fn duration(&self) -> Duration;
}
impl AnySection for Section {
	fn name(&self) -> &str {
		&self.name
	}
	fn progress(&self) -> Option<(u64, u64)> {
		self.progress
	}
	fn duration(&self) -> Duration {
		self.start.elapsed().unwrap()
	}
}
impl AnySection for FinishedSection {
	fn name(&self) -> &str {
		&self.name
	}
	fn progress(&self) -> Option<(u64, u64)> {
		self.progress
	}
	fn duration(&self) -> Duration {
		self.duration
	}
}
#[derive(Debug)]
pub struct TaskHandle<T> {
	rx: mpsc::Receiver<TaskEvent>,
	thread: Option<thread::JoinHandle<T>>,
	name: Box<str>,
	section: Option<Section>,
	finished: Vec<FinishedSection>,
}

impl<T> TaskHandle<T> {
	pub fn name(&self) -> &str {
		&self.name
	}
	pub fn section(&self) -> Option<&Section> {
		self.section.as_ref()
	}
	fn process_event(&mut self, event: TaskEvent) {
		match event {
			TaskEvent::Begin(name, progress) => {
				let now = SystemTime::now();
				if let Some(Section {
					name,
					progress,
					start,
				}) = self.section.replace(Section {
					name,
					progress: progress.map(|v| (0, v)),
					start: now,
				}) {
					self.finished.push(FinishedSection {
						name,
						progress,
						duration: now.duration_since(start).unwrap_or(Duration::from_secs(0)),
					});
				}
			}
			TaskEvent::End => {
				if let Some(Section {
					name,
					progress,
					start,
				}) = self.section.take()
				{
					self.finished.push(FinishedSection {
						name,
						progress,
						duration: SystemTime::now()
							.duration_since(start)
							.unwrap_or(Duration::from_secs(0)),
					});
				}
			}
			TaskEvent::Progress(step) => {
				if let Some(Section {
					progress: Some((progress, total)),
					..
				}) = &mut self.section
				{
					*progress = (*progress + step).min(*total);
				}
			}
		}
	}
	pub fn update_raw<F: FnOnce(&mut mpsc::Receiver<TaskEvent>) -> Result<TaskEvent, bool>>(
		&mut self,
		f: F,
	) -> bool {
		// if only unwrap_err_or
		match f(&mut self.rx).map(|event| {
			self.process_event(event);
			while let Ok(event) = self.rx.try_recv() {
				self.process_event(event);
			}
		}) {
			Ok(_) => true,
			Err(v) => v,
		}
	}
	pub fn update_blocking(&mut self) -> bool {
		self.update_raw(|ch| ch.recv().map_err(|_| false))
	}
	pub fn update(&mut self) -> bool {
		self.update_raw(|ch| {
			ch.try_recv()
				.map_err(|e| matches!(e, mpsc::TryRecvError::Empty))
		})
	}
	pub fn take_finished(&mut self) -> Vec<FinishedSection> {
		std::mem::take(&mut self.finished)
	}
	pub fn finish(mut self) -> T {
		self.thread.take().unwrap().join().unwrap()
	}
}

pub struct TaskContext {
	tx: mpsc::Sender<TaskEvent>,
}

// if the parent drops the handle then we just ignore the error
impl TaskContext {
	pub fn begin(&self, label: impl Into<String>, progress: Option<u64>) {
		drop(
			self.tx
				.send(TaskEvent::Begin(label.into().into_boxed_str(), progress)),
		);
	}
	/// end a section without starting a new one
	pub fn end(&self) {
		drop(self.tx.send(TaskEvent::End));
	}
	pub fn step(&self, n: u64) {
		if n > 0 {
			drop(self.tx.send(TaskEvent::Progress(n)));
		}
	}
}

/// spawn a task and return a handle, data must be 'static to prevent use-after-free (trust me I tried)
pub fn task<R: Send + 'static, F: FnOnce(TaskContext) -> R + Send + 'static>(
	name: impl Into<String>,
	f: F,
) -> TaskHandle<R> {
	let (tx, rx) = mpsc::channel();
	let name = name.into();
	let thread = std::thread::Builder::new()
		.name(format!("task {name:?}"))
		.spawn(|| f(TaskContext { tx }))
		.unwrap();
	let name = name.into_boxed_str();
	TaskHandle {
		rx,
		thread: Some(thread),
		name,
		finished: Vec::new(),
		section: None,
	}
}

#[cfg(test)]
mod tests {
	use super::{render, task};
	fn sleep(n: u64) {
		std::thread::sleep(std::time::Duration::from_millis(n));
	}
	#[test]
	fn success() {
		let handle = task("test success", |ctx| {
			sleep(10);
			ctx.begin("1", Some(60));
			for i in 0..=10 {
				sleep(10);
				ctx.step(i);
			}
			sleep(10);
			ctx.begin("2", Some(55));
			for i in 0..=10 {
				sleep(10);
				ctx.step(i);
			}
			sleep(10);
			ctx.begin("3", None);
			sleep(10);
		});
		render::render_term(handle);
	}
	#[test]
	fn ub_hell() {
		use std::sync::Arc;
		use std::sync::atomic::{AtomicUsize, Ordering};
		let value = Arc::new(AtomicUsize::new(0));
		println!("{value:?}");
		let v = value.clone();
		// this requires 'static so no ub hell
		let handle = task("test ub_hell", move |_ctx| {
			sleep(100);
			v.store(1, Ordering::Relaxed);
		});
		while value.load(Ordering::Relaxed) == 0 {
			print!("{value:?}");
			sleep(10);
		}
		println!("{value:?}");
		handle.finish();
		println!("{value:?}");
	}
}
