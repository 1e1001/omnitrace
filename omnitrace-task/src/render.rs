//! built-in renderers for basic uses, all look like `Fn(TaskHandle<T>) -> T`
use crate::{AnySection, TaskHandle};
use std::time::{Duration, Instant};

/// debugging renderer (event logger to stdout)
pub fn render_debug<T>(mut task: TaskHandle<T>) -> T {
	println!("{}: Start", task.name());
	while task.update_blocking() {
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

fn per_mille(progress: (u64, u64)) -> u64 {
	const C1000: u64 = u64::MAX / 1000;
	const C100: u64 = u64::MAX / 100;
	const C10: u64 = u64::MAX / 10;
	match progress.0 {
		..=C1000 => (progress.0 * 1000) / progress.1,
		..=C100 => (progress.0 * 100) / (progress.1 / 10),
		..=C10 => (progress.0 * 10) / (progress.1 / 1000),
		_ => progress.0 / (progress.1 / 1000),
	}
}

pub fn render_term<T>(mut task: TaskHandle<T>) -> T {
	// █▉▊▋▌▍▎▏
	const BOX_WIDTH: usize = '█'.len_utf8();
	const BAR_EMPTY: &str = "                        ";
	const BAR_FULL: &str = "████████████████████████";
	const BAR_PROGRESS: &str = " ▏▎▍▌▋▊▉█";
	const SPIN_ANIM: &str = " ▏▎▍▌▋▊▉█▇▆▅▄▃▂▁";
	const TIME_STEP: Duration = Duration::from_millis(100);
	fn render_bar(section: Option<&impl AnySection>, counter: u8) {
		if let Some(section) = section {
			print!(" ▐ {}\t", section.name());
			if let Some(progress) = section.progress() {
				if progress.0 == progress.1 {
					print!(" 100% ▌{BAR_FULL}█▐");
				} else {
					let pm = per_mille(progress) as usize;
					let steps = pm / 5;
					let chars = steps >> 3;
					let steps = steps & 7;
					print!(
						"{:>2}.{}% ▌{}{}{}▐",
						pm / 10,
						pm % 10,
						&BAR_FULL[0..chars * BOX_WIDTH],
						&BAR_PROGRESS[(1 + steps * BOX_WIDTH).saturating_sub(BOX_WIDTH)
							..1 + steps * BOX_WIDTH],
						&BAR_EMPTY[0..24 - chars]
					);
				}
			} else {
				let counter = counter as usize;
				print!(
					"▌{}▐",
					&SPIN_ANIM[(1 + counter * BOX_WIDTH).saturating_sub(BOX_WIDTH)
						..1 + counter * BOX_WIDTH]
				);
			}
			print!(" {:?}", section.duration());
		}
		println!("\x1b[K");
	}
	let mut counter = 0;
	let mut next_frame = Instant::now();
	println!("{}:", task.name());
	render_bar(task.section(), counter);
	while task.update_raw(|rx| {
		rx.recv_timeout(next_frame.duration_since(Instant::now()))
			.map_err(|e| matches!(e, std::sync::mpsc::RecvTimeoutError::Timeout))
	}) {
		let now = Instant::now();
		while next_frame <= now {
			next_frame += TIME_STEP;
			counter += 1;
			counter &= 15;
		}
		print!("\x1b[A");
		for finished in task.take_finished() {
			render_bar(Some(&finished), counter);
		}
		render_bar(task.section(), counter);
	}
	println!("\x1b[A▄▟");
	task.finish()
}
