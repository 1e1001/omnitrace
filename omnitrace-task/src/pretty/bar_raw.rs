//! raw progress bar api

use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
use std::sync::{mpsc, RwLock};
use std::time::{Duration, Instant};

use fxhash::FxHashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BarId(u32);
impl BarId {
	pub fn new() -> Self {
		static NEXT_ID: AtomicU32 = AtomicU32::new(0);
		BarId(NEXT_ID.fetch_add(1, AtomicOrdering::Relaxed))
	}
}

#[derive(Debug, Clone, Copy)]
pub struct UpdateEvent {
	pub id: BarId,
	pub permille: u16,
	pub decrement: bool,
}

#[derive(Debug, Clone)]
pub enum BarEvent {
	New {
		id: BarId,
		name: String,
		time: Instant,
		progress: bool,
	},
	Close {
		id: BarId,
		time: Instant,
		mark: bool,
		callback: mpsc::SyncSender<()>,
	},
	Check,
}

static THREAD_SENDER: RwLock<Option<(mpsc::SyncSender<BarEvent>, mpsc::SyncSender<UpdateEvent>)>> =
	RwLock::new(None);

pub fn send_bar_event(evt: BarEvent) {
	if THREAD_SENDER
		.read()
		.unwrap()
		.as_ref()
		.expect("Progress bar thread not running")
		.0
		.send(evt)
		.is_err()
	{
		panic!("Progress bar thread not running");
	}
}
pub fn send_update_event(evt: UpdateEvent) {
	if let Err(mpsc::TrySendError::Disconnected(_)) = THREAD_SENDER
		.read()
		.unwrap()
		.as_ref()
		.expect("Progress bar thread not running")
		.1
		.try_send(evt)
	{
		panic!("Progress bar thread not running");
	}
}

// singleton in rust :)
pub fn load_bars() {
	loop {
		let lock = THREAD_SENDER.read().unwrap();
		if let Some((old, _)) = &*lock {
			if old.try_send(BarEvent::Check).is_err() {
				drop(lock);
				continue;
			}
		} else {
			drop(lock);
			let mut lock = THREAD_SENDER.write().unwrap();
			if lock.is_none() {
				let (bar_tx, bar_rx) = mpsc::sync_channel(16);
				let (up_tx, up_rx) = mpsc::sync_channel(64);
				*lock = Some((bar_tx, up_tx));
				crate::pretty::thread::spawn("Progress Bars", || {
					progress_bar_thread(bar_rx, up_rx);
					*THREAD_SENDER.write().unwrap() = None;
				});
			}
		}
		return;
	}
}

enum BarEntryKind {
	Progress(u16),
	Spinner(u8),
}
struct BarEntry {
	id: BarId,
	name: String,
	time: Instant,
	kind: BarEntryKind,
}
fn print_bar(bar: &BarEntry, now: Instant) {
	eprint!("{}\t", bar.name);
	match bar.kind {
		BarEntryKind::Progress(permille) => {
			let mut full_count = (permille / 40) as usize;
			eprint!(
				" {:>3}.{}% ▌{}",
				permille / 10,
				permille % 10,
				&FULL_STR[0..3 * full_count]
			);
			if permille < 1000 {
				let idx = 3 * ((permille % 40) as usize / 5);
				eprint!("{}", &PROGRESS_STR[idx.saturating_sub(2)..idx + 1]);
				full_count += 1;
			}
			eprint!("{}", &SPACE_STR[0..(PROGRESS_LEN - full_count)]);
		}
		BarEntryKind::Spinner(frame) => {
			let idx = (frame * 3) as usize;
			eprint!("▌{}", &SPINNER_STR[idx.saturating_sub(2)..idx + 1]);
		}
	}
	eprint!("▐ ");
	let duration = now.duration_since(bar.time);
	let tenths = duration.subsec_millis() / 100;
	let secs = duration.as_secs();
	let (mins, secs) = (secs / 60, secs % 60);
	let (hours, mins) = (mins / 60, mins % 60);
	match (hours, mins) {
		(1.., _) => eprint!("{hours}:{mins:>02}:{secs:>02}.{tenths}"),
		(0, 1..) => eprint!("{mins}:{secs:>02}.{tenths}"),
		(0, 0) => eprint!("{secs}.{tenths}s"),
	}
	eprintln!("\x1b[K");
}

const SPINNER_STR: &str = " ▏▎▍▌▋▊▉█▇▆▅▄▃▂▁";
const PROGRESS_STR: &str = " ▏▎▍▌▋▊▉";
const SPACE_STR: &str = "                                        ";
const FULL_STR: &str = "█████████████████████████";
const TIME_STEP: Duration = Duration::from_millis(100);
const SPINNER_LEN: u8 = 16;
const PROGRESS_LEN: usize = 25;

fn progress_bar_thread(bar_rx: mpsc::Receiver<BarEvent>, up_rx: mpsc::Receiver<UpdateEvent>) {
	let mut bars = FxHashMap::default();
	let mut order = Vec::new();
	let mut last_frame = Instant::now();
	let mut ret = None::<u8>;
	loop {
		if let Some(ret) = &mut ret {
			*ret -= 1;
			if *ret == 0 {
				return;
			}
		}
		loop {
			match bar_rx.recv_timeout(TIME_STEP / 2) {
				Ok(BarEvent::New {
					id,
					name,
					time,
					progress,
				}) => {
					let order_id = order.len();
					bars.insert(id, order_id);
					order.push(BarEntry {
						id,
						time,
						kind: if progress {
							eprintln!("{}\t   0.0% ▌{}▐ 0s", name, &SPACE_STR[0..PROGRESS_LEN]);
							BarEntryKind::Progress(0)
						} else {
							eprintln!("{}\t▌ ▐ 0s", name);
							BarEntryKind::Spinner(0)
						},
						name,
					});
				}
				Ok(BarEvent::Close {
					id,
					time,
					mark,
					callback,
				}) => {
					if let Some(v) = bars.remove(&id) {
						let mut bar = order.remove(v);
						for below in order.iter_mut().skip(v) {
							bars.entry(below.id).and_modify(|v| *v -= 1);
						}
						if mark {
							match &mut bar.kind {
								BarEntryKind::Progress(permille) => *permille = 1000,
								BarEntryKind::Spinner(frame) => *frame = 0,
							}
							eprint!("\x1b[{}A", order.len() + 1);
							print_bar(&bar, time);
							if order.len() > 0 {
								eprint!("\x1b[{}B", order.len());
							}
						} else {
							eprint!("\x1b[A\x1b[K");
						}
						if order.len() == 0 {
							drop(std::io::Write::flush(&mut std::io::stderr()));
							drop(callback.send(()));
							ret = Some(2);
						}
					}
					drop(callback.send(()));
				}
				Ok(BarEvent::Check) => ret = None,
				Err(mpsc::RecvTimeoutError::Timeout) => break,
				Err(mpsc::RecvTimeoutError::Disconnected) => panic!("Wrong order disconnect!"),
			}
		}
		loop {
			match up_rx.try_recv() {
				Ok(UpdateEvent {
					id,
					permille,
					decrement,
				}) => {
					if let Some(v) = bars.get(&id) {
						if let BarEntryKind::Progress(a) = &mut order[*v].kind {
							*a = permille.clamp(if decrement { 0 } else { *a }, 1000);
						}
					}
				}
				Err(mpsc::TryRecvError::Empty) => break,
				Err(mpsc::TryRecvError::Disconnected) => panic!("Wrong order disconnect!"),
			}
		}
		let now = Instant::now();
		if now.duration_since(last_frame) > TIME_STEP {
			last_frame = now;
			if order.len() > 0 {
				eprint!("\x1b[{}A", order.len());
			}
			for bar in &mut order {
				if let BarEntryKind::Spinner(frame) = &mut bar.kind {
					*frame = (*frame + 1) % SPINNER_LEN;
				}
				print_bar(&*bar, now);
			}
		}
		//std::thread::sleep(TIME_STEP.saturating_sub(last_frame.elapsed()) / 2);
	}
}
