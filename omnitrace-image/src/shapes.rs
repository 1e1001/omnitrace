use omnitrace::prelude::*;
use omnitrace_math::prelude::*;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Circle<T>(pub T);
impl<T: Real> Trace<Vec2<T>, T> for Circle<T> {
	type Cache = ();
	fn trace(&self, input: Vec2<T>, _cache: &mut Self::Cache) -> T {
		input.magnitude() - self.0
	}
}
