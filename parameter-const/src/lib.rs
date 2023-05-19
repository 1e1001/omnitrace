use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::thread::LocalKey;

pub mod __ {
	pub use paste::paste;
	pub use std::cell::{Cell, RefCell};
	pub use std::rc::Rc;
	use std::thread::LocalKey;
	pub const fn rc_param<T>(k: &'static LocalKey<Rc<RefCell<T>>>) -> crate::RcParameter<T> {
		crate::RcParameter { k }
	}
	pub const fn copy_param<T: Copy>(k: &'static LocalKey<Cell<T>>) -> crate::CopyParameter<T> {
		crate::CopyParameter { k }
	}
}

#[derive(Debug)]
pub struct RcParameter<T: 'static> {
	pub(crate) k: &'static LocalKey<Rc<RefCell<T>>>,
}

impl<T> RcParameter<T> {
	pub fn get(&self) -> Rc<RefCell<T>> {
		self.k.with(|v| v.clone())
	}
	pub fn with<R, F: FnOnce() -> R>(&self, v: T, f: F) -> R {
		let old = std::mem::replace(&mut *self.get().borrow_mut(), v);
		let res = f();
		drop(std::mem::replace(&mut *self.get().borrow_mut(), old));
		res
	}
}

#[derive(Debug)]
pub struct CopyParameter<T: 'static + Copy> {
	pub(crate) k: &'static LocalKey<Cell<T>>,
}

impl<T: Copy> CopyParameter<T> {
	pub fn get(&self) -> T {
		self.k.with(|v| v.get())
	}
	pub fn with<R, F: FnOnce() -> R>(&self, v: T, f: F) -> R {
		let old = self.k.with(|o| o.replace(v));
		let res = f();
		self.k.with(|o| o.set(old));
		res
	}
}

#[macro_export]
macro_rules! define {
	($(#[$meta:meta])* $vis:vis rc_parameter $id:ident: $ty:ty = $init:expr) => {
		$crate::__::paste! {
			thread_local! {
				static [<__INNER_ $id>]: $crate::__::Rc<$crate::__::RefCell<$ty>> = $crate::__::Rc::new($crate::__::RefCell::new($init));
			}
			$(#[$meta])*
			$vis static $id: $crate::Parameter<$ty> = $crate::__::rc_param(&[<__INNER_ $id>]);
		}
	};
	($(#[$meta:meta])* $vis:vis copy_parameter $id:ident: $ty:ty = $init:expr) => {
		$crate::__::paste! {
			thread_local! {
				static [<__INNER_ $id>]: $crate::__::Cell<$ty> = $crate::__::Cell::new($init);
			}
			$(#[$meta])*
			$vis static $id: $crate::CopyParameter<$ty> = $crate::__::copy_param(&[<__INNER_ $id>]);
		}
	};
	($(
		$(#[$meta:meta])* $vis:vis $mode:ident $id:ident: $ty:ty = $init:expr;
	)*) => {
		$($crate::define!(
			$(#[$meta])* $vis $mode $id: $ty = $init
		);)*
	};
}
