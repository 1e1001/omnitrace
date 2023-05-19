//! high-level progress bar api

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::Instant;

use super::bar_raw as raw;

fn into_permille(current: u64, total: u64) -> u16 {
	const CUTOFF: u64 = u64::MAX / 1000;
	if current >= total {
		1000
	} else if current > CUTOFF {
		(current / (total / 1000)) as u16
	} else {
		((current * 1000) / total) as u16
	}
}

#[derive(Debug)]
pub struct Bar {
	id: raw::BarId,
	current: AtomicU64,
	total: Option<u64>,
	finish: bool,
}
impl Bar {
	pub fn new(name: impl Into<String>, total: Option<u64>) -> Self {
		raw::load_bars();
		let id = raw::BarId::new();
		raw::send_bar_event(raw::BarEvent::New {
			id,
			name: name.into(),
			time: Instant::now(),
			progress: total.is_some(),
		});
		Self {
			id,
			current: AtomicU64::new(0),
			total,
			finish: true,
		}
	}
	#[cfg_attr(feature = "inline", inline(always))]
	pub fn increment(&self, n: u64) {
		if let Some(total) = self.total {
			raw::send_update_event(raw::UpdateEvent {
				id: self.id,
				permille: into_permille(self.current.fetch_add(n, Ordering::Relaxed) + n, total),
				decrement: false,
			});
		}
	}
	#[cfg_attr(feature = "inline", inline(always))]
	pub fn finish_on_drop(mut self, v: bool) -> Self {
		self.finish = v;
		self
	}
}
impl Drop for Bar {
	#[cfg_attr(feature = "inline", inline(always))]
	fn drop(&mut self) {
		let (tx, rx) = mpsc::sync_channel(0);
		raw::send_bar_event(raw::BarEvent::Close {
			id: self.id,
			time: Instant::now(),
			mark: self.finish,
			callback: tx,
		});
		rx.recv().unwrap();
	}
}
