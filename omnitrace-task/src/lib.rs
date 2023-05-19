use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, Duration};

pub mod render;

enum TaskEvent {
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
#[derive(Debug)]
pub struct TaskHandle<T> {
	rx: mpsc::Receiver<TaskEvent>,
	thread: Option<thread::JoinHandle<T>>,
	name: Box<str>,
	section: Section,
}

impl<T> TaskHandle<T> {
	pub fn name(&self) -> &str {
		&self.name
	}
	pub fn section(&self) -> &Section {
		&self.section
	}
	pub fn pop_section(&mut self) -> Option<FinishedSection> {

	}
	// fn process_event(&mut self, event: TaskEvent, stack: &mut Option<Section>) {
	// 	match event {
	// 		TaskEvent::Begin(name, size) => {
	// 			*stack = std::mem::replace(&mut self.section, Some(Section {
	// 				name,
	// 				progress: size.map(|v| (0, v)),
	// 				start: SystemTime::now(),
	// 			}));
	// 		}
	// 		TaskEvent::Progress(step) => {
	// 			if let Some(Section { progress: Some((val, len)), .. }) = &mut self.section {
	// 				*val = (*len).min(*val + step);
	// 			}
	// 		}
	// 	}
	// }
	// fn recv_update_part2(&mut self, mut stack: Option<Section>) -> TaskUpdate {
	// 	// don't return on error here since we've already got a partial event, only return [`None`] on the next event.
	// 	while let Ok(event) = self.rx.try_recv() {
	// 		self.process_event(event, &mut stack);
	// 	}
	// 	TaskUpdate {
	// 		section: self.section.as_ref().unwrap(),
	// 		old_section: stack,
	// 	}
	// }
	// pub fn recv_update(&mut self) -> Option<TaskUpdate> {
	// 	// always block for one event, but collect as many as are available
	// 	let mut stack = None;
	// 	self.process_event(self.rx.recv().ok()?, &mut stack);
	// 	Some(self.recv_update_part2(stack))
	// }
	// pub fn try_recv_update(&mut self) -> Result<TaskUpdate, mpsc::TryRecvError> {
	// 	let mut stack = None;
	// 	self.process_event(self.rx.recv()?, &mut stack);
	// 	Ok(self.recv_update_part2(stack))
	// }
	// /// will panic unless [`recv_event`] returned None
	// pub fn finish(mut self) -> T {
	// 	if let Err(mpsc::TryRecvError::Disconnected) = self.rx.try_recv() {
	// 		// set a flag here to not panic on drop?
	// 		self.thread.take().unwrap().join().unwrap()
	// 	} else {
	// 		panic!("TaskHandle::finish called on running task");
	// 	}
	// }
}
impl<T> Drop for TaskHandle<T> {
	fn drop(&mut self) {
		if self.thread.is_some() {
			panic!("TaskHandle dropped without completing");
		}
	}
}

pub struct TaskContext {
	tx: mpsc::SyncSender<TaskEvent>,
}

// if the parent drops the handle then we just ignore the error
impl TaskContext {
	pub fn begin(&self, label: impl Into<String>, progress: Option<u64>) {
		drop(self.tx.send(TaskEvent::Begin(label.into().into_boxed_str(), progress)));
	}
	/// end a section without starting a new one
	pub fn end(&self) {
		drop(self.tx.send(TaskEvent::Begin(Box::new(""), None)));
	}
	pub fn step(&self, n: u64) {
		if n > 0 {
			drop(self.tx.send(TaskEvent::Progress(n)));
		}
	}
}

pub fn task<R: Send + 'static, F: FnOnce(TaskContext) -> R + Send + 'static>(name: impl Into<String>, f: F) -> TaskHandle<R> {
	let (tx, rx) = mpsc::sync_channel(128);
	let name = name.into();
	let thread = std::thread::Builder::new().name(format!("task {name:?}")).spawn(|| {
		f(TaskContext { tx })
	}).unwrap();
	let name = name.into_boxed_str();
	TaskHandle {
		rx,
		thread: Some(thread),
		name: name.clone(),
		section: Some(Section {
			name: name,
			progress: None,
			start: SystemTime::now(),
		}),
	}
}

#[cfg(test)]
mod tests {
	use super::{task, render};
	fn sleep(n: u64) {
		std::thread::sleep(std::time::Duration::from_millis(n));
	}
	#[test]
	fn success() {
		let handle = task("test success", |ctx| {
			sleep(10);
			ctx.begin("1", Some(55));
			for i in 0..=10 {
				sleep(10);
				ctx.step(i);
			}
			sleep(10);
			ctx.begin("2", Some(60));
			for i in 0..=10 {
				sleep(10);
				ctx.step(i);
			}
			sleep(10);
			ctx.begin("3", None);
			sleep(10);
		});
		render::render_debug(handle);
	}
	#[test]
	#[should_panic]
	fn early_exit() {
		let _handle = task("test success", |ctx| {
			sleep(100);
		});
	}
}
