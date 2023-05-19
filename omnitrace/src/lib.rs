use std::fmt;
use std::marker::PhantomData;

use pretty::bar::Bar;

pub mod ext;

pub mod prelude {
	pub use crate::ext::TraceExtCore;
	pub use crate::{Func, In2Out, Trace};
}

pub struct PhantomNothing<T>(PhantomData<*const T>);
impl<T> PhantomNothing<T> {
	#[cfg_attr(feature = "inline", inline(always))]
	const fn new() -> Self {
		Self(PhantomData)
	}
}
impl<T> fmt::Debug for PhantomNothing<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str("PhantomNothing")
	}
}
impl<T> Clone for PhantomNothing<T> {
	fn clone(&self) -> Self {
		Self::new()
	}
}
impl<T> Copy for PhantomNothing<T> {}

pub trait Trace<I, O> {
	type Cache: Default;
	fn trace(&self, input: I, cache: &mut Self::Cache) -> O;
}

impl<I, O, T: Trace<I, O>> Trace<I, O> for &T {
	type Cache = T::Cache;
	fn trace(&self, input: I, cache: &mut Self::Cache) -> O {
		Trace::trace(*self, input, cache)
	}
}

macro_rules! impl_tuple_trace {
	(($($n:tt)*)) => {
		paste::paste! {
			impl<$([<I $n>], [<O $n>], [<T $n>]: Trace<[<I $n>], [<O $n>]>),*> Trace<($([<I $n>],)*), ($([<O $n>],)*)> for ($([<T $n>],)*) {
				type Cache = ($([<T $n>]::Cache,)*);
				fn trace(&self, input: ($([<I $n>],)*), cache: &mut Self::Cache) -> ($([<O $n>],)*) {
					($(self.$n.trace(input.$n, &mut cache.$n),)*)
				}
			}
		}
	};
	($($t:tt)*) => { $(impl_tuple_trace!($t);)* };
}

impl_tuple_trace! {
	(0)
	(0 1)
	(0 1 2)
	(0 1 2 3)
	(0 1 2 3 4)
	(0 1 2 3 4 5)
	(0 1 2 3 4 5 6)
	(0 1 2 3 4 5 6 7)
	(0 1 2 3 4 5 6 7 8)
	(0 1 2 3 4 5 6 7 8 9)
	(0 1 2 3 4 5 6 7 8 9 10)
	(0 1 2 3 4 5 6 7 8 9 10 11)
}

#[derive(Debug, Clone, Copy)]
pub struct In2Out;
impl<T> Trace<T, T> for In2Out {
	type Cache = ();
	fn trace(&self, input: T, _cache: &mut Self::Cache) -> T {
		input
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Func<F>(pub F);

impl<I, O, F: Fn(I) -> O> Trace<I, O> for Func<F> {
	type Cache = ();
	fn trace(&self, input: I, _cache: &mut Self::Cache) -> O {
		(self.0)(input)
	}
}

pub fn iterate_linear<I, O, V, T>(name: impl Into<String>, i: V, s: T) -> Vec<O>
where
	V: Iterator<Item = I>,
	T: Trace<I, O>,
{
	let mut cache = T::Cache::default();
	let bar = Bar::new(name, i.size_hint().1.and_then(|v| u64::try_from(v).ok()));
	i.map(|input| {
		let output = s.trace(input, &mut cache);
		bar.increment(1);
		output
	})
	.collect()
}

#[cfg(feature = "parallel")]
pub fn iterate_parallel<I, O, V, T>(name: impl Into<String>, i: V, s: T) -> Vec<O>
where
	I: Send,
	O: Send,
	V: Iterator<Item = I> + Send,
	T: Trace<I, O> + Sync,
	T::Cache: Send,
{
	use rayon::iter::{ParallelBridge, ParallelIterator};
	let bar = Bar::new(name, i.size_hint().1.and_then(|v| u64::try_from(v).ok()));
	i.par_bridge()
		.map_init(T::Cache::default, |cache, input| {
			let output = s.trace(input, cache);
			bar.increment(1);
			output
		})
		.collect()
}

#[cfg(feature = "parallel")]
pub fn iterate_parallel_fast<I, O, V, T>(name: impl Into<String>, len: u64, i: V, s: T) -> Vec<O>
where
	I: Send,
	O: Send,
	V: rayon::iter::ParallelIterator<Item = I> + Send,
	T: Trace<I, O> + Sync,
	T::Cache: Send,
{
	use rayon::iter::ParallelIterator;
	let bar = Bar::new(name, len);
	i.map_init(T::Cache::default, |cache, input| {
		let output = s.trace(input, cache);
		bar.increment(1);
		output
	})
	.collect()
}

#[cfg_attr(feature = "inline", inline(always))]
pub fn iterate<I, O, V, T>(name: impl Into<String>, i: V, s: T) -> Vec<O>
where
	I: Send,
	O: Send,
	V: Iterator<Item = I> + Send,
	T: Trace<I, O> + Sync,
	T::Cache: Send,
{
	#[cfg(feature = "parallel")]
	return iterate_parallel(name, i, s);
	#[cfg(not(feature = "parallel"))]
	return iterate_linear(name, i, s);
}
