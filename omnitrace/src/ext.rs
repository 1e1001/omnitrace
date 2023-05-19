use crate::{PhantomNothing, Trace};

#[derive(Debug, Clone, Copy)]
pub struct WrapLazy<T, F> {
	parent: T,
	func: F,
}
impl<I, O, R: Trace<I, O>, F: Fn(T) -> R, T: Clone> Trace<I, O> for WrapLazy<T, F> {
	type Cache = (Option<R>, R::Cache);
	fn trace(&self, input: I, cache: &mut Self::Cache) -> O {
		cache
			.0
			.get_or_insert_with(|| (self.func)(self.parent.clone()))
			.trace(input, &mut cache.1)
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Cast<T, I, O> {
	parent: T,
	_phantom: PhantomNothing<(I, O)>,
}
impl<I, O, T: Trace<I, O>> Trace<I, O> for Cast<T, I, O> {
	type Cache = T::Cache;
	fn trace(&self, input: I, cache: &mut Self::Cache) -> O {
		self.parent.trace(input, cache)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct MapIn<T, F, P> {
	parent: T,
	func: F,
	_phantom: PhantomNothing<P>,
}
impl<I, O, I2, F: Fn(I2) -> I, T: Trace<I, O>> Trace<I2, O> for MapIn<T, F, I> {
	type Cache = T::Cache;
	fn trace(&self, input: I2, cache: &mut Self::Cache) -> O {
		self.parent.trace((self.func)(input), cache)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct MapOut<T, F, P> {
	parent: T,
	func: F,
	_phantom: PhantomNothing<P>,
}
impl<I, O, O2, F: Fn(O) -> O2, T: Trace<I, O>> Trace<I, O2> for MapOut<T, F, O> {
	type Cache = T::Cache;
	fn trace(&self, input: I, cache: &mut Self::Cache) -> O2 {
		(self.func)(self.parent.trace(input, cache))
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Out2In<T, U, P> {
	parent: T,
	then: U,
	_phantom: PhantomNothing<P>,
}
impl<I, O, O2, U: Trace<O, O2>, T: Trace<I, O>> Trace<I, O2> for Out2In<T, U, O> {
	type Cache = (T::Cache, U::Cache);
	fn trace(&self, input: I, cache: &mut Self::Cache) -> O2 {
		self.then
			.trace(self.parent.trace(input, &mut cache.0), &mut cache.1)
	}
}

pub trait TraceExtCore<I, O>: Trace<I, O> {
	#[cfg_attr(feature = "inline", inline(always))]
	fn by_ref(&self) -> &Self
	where
		Self: Sized,
	{
		self
	}
	#[cfg_attr(feature = "inline", inline(always))]
	fn wrap<R, F: FnOnce(Self) -> R>(self, f: F) -> R
	where
		Self: Sized,
	{
		f(self)
	}
	#[cfg_attr(feature = "inline", inline(always))]
	fn wrap_lazy<R, F: Fn(Self) -> R>(self, f: F) -> WrapLazy<Self, F>
	where
		Self: Sized,
	{
		WrapLazy {
			parent: self,
			func: f,
		}
	}
	#[cfg_attr(feature = "inline", inline(always))]
	fn cast<I2, O2>(self) -> Cast<Self, I, O>
	where
		Self: Sized + Trace<I2, O2>,
		fn(I2, O2): Fn(I, O),
	{
		Cast {
			parent: self,
			_phantom: PhantomNothing::new(),
		}
	}
	#[cfg_attr(feature = "inline", inline(always))]
	fn map_in<I2, F: Fn(I2) -> I>(self, f: F) -> MapIn<Self, F, I>
	where
		Self: Sized,
	{
		MapIn {
			parent: self,
			func: f,
			_phantom: PhantomNothing::new(),
		}
	}
	#[cfg_attr(feature = "inline", inline(always))]
	fn map_out<O2, F: Fn(O) -> O2>(self, f: F) -> MapOut<Self, F, O>
	where
		Self: Sized,
	{
		MapOut {
			parent: self,
			func: f,
			_phantom: PhantomNothing::new(),
		}
	}
	#[cfg_attr(feature = "inline", inline(always))]
	fn out2in<O2, T: Trace<O, O2>>(self, s: T) -> Out2In<Self, T, O>
	where
		Self: Sized,
	{
		Out2In {
			parent: self,
			then: s,
			_phantom: PhantomNothing::new(),
		}
	}
}
impl<I, O, T: Trace<I, O>> TraceExtCore<I, O> for T {}
