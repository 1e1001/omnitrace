//! built-in renderers for basic uses, all look like `Fn(TaskHandle<T>) -> T`
use crate::TaskHandle;

/// debugging renderer (event logger to stdout)
pub fn render_debug<T>(mut task: TaskHandle<T>) -> T {
	println!("{}: Start", task.name());
	while task.block() {
		if let Some(section) = task.section() {
			print!("{}: {}", task.name(), section.name);
			if let Some((now, len)) = section.progress {
				println!(" {now}/{len}");
			} else {
				println!();
			}
		}
	}
	println!("{}: Done", task.name());
	task.finish()
}
