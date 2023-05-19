use omnitrace::prelude::*;
use omnitrace_math::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Trans<T, V> {
	parent: T,
	backward_f64: Mat2<f64>,
	backward: Mat3<V>,
}
impl<V: Real + MulAdd<Output = V>, O, T: Trace<Vec2<V>, O>> Trace<Vec2<V>, O> for Trans<T, V> {
	type Cache = T::Cache;
	fn trace(&self, input: Vec2<V>, cache: &mut Self::Cache) -> O {
		let old = crate::SCALE.get();
		crate::SCALE.with(old * self.backward_f64, || {
			self.parent.trace(self.backward.mul_point_2d(input), cache)
		})
	}
}

pub trait TraceExtVec2Transform<V: Real + MulAdd<Output = V>, O>: Trace<Vec2<V>, O> {
	fn trans(self, m: Mat3<V>) -> Trans<Self, V>
	where
		Self: Sized,
	{
		let inverse = Mat3::from(Mat4::from(m).inverted());
		Trans {
			parent: self,
			// FIXME: better way to get from `impl Real` to `f64`?
			backward_f64: Mat2::from(inverse).map(|v| v.to_f64().unwrap()),
			backward: inverse,
		}
	}
}
impl<V: Real + MulAdd<Output = V>, O, T: Trace<Vec2<V>, O>> TraceExtVec2Transform<V, O> for T {}

// #[derive(Debug, Clone, Copy)]
// pub struct StackOver<T, U> {
// 	parent: T,
// 	top: U,
// }
// impl<I: Clone, T: Trace<I, Color>, U: Trace<I, Color>> Trace<I, Color> for StackOver<T, U> {
// 	type Cache = (T::Cache, U::Cache);
// 	fn trace(&self, input: I, cache: &mut Self::Cache) -> Color {
// 		self.top
// 			.trace(input.clone(), &mut cache.1)
// 			.over(self.parent.trace(input, &mut cache.0))
// 	}
// }
//
// pub trait TraceExtCompositing<I>: Trace<I, Color> {
// 	fn stack_over<T: Trace<I, Color>>(self, top: T) -> StackOver<Self, T>
// 	where
// 		Self: Sized,
// 	{
// 		StackOver { parent: self, top }
// 	}
// }
// impl<I, T: Trace<I, Color>> TraceExtCompositing<I> for T {}

macro_rules! impl_ext_compositing {
	($($then_blend:ident $blend:ident $struct:ident $base:tt :: $func:tt)*) => {
		$(
			#[derive(Debug, Clone, Copy)]
			pub struct $struct<T, U> {
				top: T,
				bottom: U,
			}
			impl<I: Clone, T: Trace<I, Color>, U: Trace<I, Color>> Trace<I, Color> for $struct<T, U> {
				type Cache = (T::Cache, U::Cache);
				fn trace(&self, input: I, cache: &mut Self::Cache) -> Color {
					$base::$func(
						self.top.trace(input.clone(), &mut cache.0),
						self.bottom.trace(input, &mut cache.1),
					)
				}
			}
		)*
		pub trait TraceExtCompositing<I>: Trace<I, Color> {$(
			fn $then_blend<T: Trace<I, Color>>(self, top: T) -> $struct<T, Self>
			where
				Self: Sized,
			{
				$struct { top, bottom: self }
			}
			fn $blend<T: Trace<I, Color>>(self, bottom: T) -> $struct<Self, T>
			where
				Self: Sized,
			{
				$struct { top: self, bottom }
			}
		)*}
		impl<I, T: Trace<I, Color>> TraceExtCompositing<I> for T {}
	};
}
impl_ext_compositing! {
	then_blend_over      blend_over      CompositeOver      Compose::over
	then_blend_inside    blend_inside    CompositeInside    Compose::inside
	then_blend_outside   blend_outside   CompositeOutside   Compose::outside
	then_blend_atop      blend_atop      CompositeAtop      Compose::atop
	then_blend_xor       blend_xor       CompositeXor       Compose::xor
	then_blend_add       blend_add       CompositeAdd       Compose::plus
	then_blend_mul       blend_mul       CompositeMul       Blend::multiply
	then_blend_screen    blend_screen    CompositeScreen    Blend::screen
	then_blend_overlay   blend_overlay   CompositeOverlay   Blend::overlay
	then_blend_darken    blend_darken    CompositeDarken    Blend::darken
	then_blend_lighten   blend_lighten   CompositeLighten   Blend::lighten
	then_blend_dodge     blend_dodge     CompositeDodge     Blend::dodge
	then_blend_burn      blend_burn      CompositeBurn      Blend::burn
	then_blend_hard      blend_hard      CompositeHard      Blend::hard_light
	then_blend_soft      blend_soft      CompositeSoft      Blend::soft_light
	then_blend_sub       blend_sub       CompositeSub       Blend::difference
	then_blend_exclusion blend_exclusion CompositeExclusion Blend::exclusion
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Sdf2Fac<T> {
	parent: T,
}
impl<V: Real + MulAdd<Output = V>, T: Trace<Vec2<V>, V>> Trace<Vec2<V>, V> for Sdf2Fac<T> {
	type Cache = T::Cache;
	fn trace(&self, input: Vec2<V>, cache: &mut Self::Cache) -> V {
		// "constants" but the one trait isn't const
		let v0 = V::zero();
		let v1 = V::one();
		let vn1 = -v1;
		let v2 = v1 + v1;
		let v1_2 = v1 / v2;
		let vn1_2 = -v1_2;
		let scale: Mat2<V> = crate::SCALE.get().map(|v| NumCast::from(v).unwrap());
		let sample_points = Vec4::new(
			Vec2::new(vn1_2, vn1_2),
			Vec2::new(vn1_2, v1_2),
			Vec2::new(v1_2, v1_2),
			Vec2::new(v1_2, vn1_2),
		);
		let sample_values = sample_points.map(|v| self.parent.trace(input + scale * v, cache));
		let samples_inside = sample_values.map(|v| v <= v0);
		let samples_inside_count = samples_inside.map(|v| if v { 1u8 } else { 0u8 }).sum();
		if samples_inside_count == 4 {
			v1
		} else {
			let sample_slopes = sample_values - sample_values.shuffled((1, 2, 3, 0));
			let sample_zeros =
				sample_values.map2(sample_slopes, |a, b| if b == v0 { v1_2 } else { a / b });
			let corner_area = (sample_zeros * sample_zeros.wxyz().map(|v| (v1 - v)) * v1_2)
				.map2(samples_inside, |a, b| if b { a } else { v0 })
				.sum();
			if sample_slopes.map(|v| v != v0).reduce_and() {
				corner_area
					+ if samples_inside_count == 2
						&& sample_zeros.map(|v| v0 <= v && v <= v1).reduce_and()
					{
						let isp = Vec4::new(
							Vec2::new(v0, v1),
							Vec2::new(v1, v0),
							Vec2::new(v0, vn1),
							Vec2::new(vn1, v0),
						)
						.map3(sample_zeros, sample_points, |a, b, c| a.mul_add(b, c));
						let isda = isp.x - isp.z;
						let isdb = isp.y - isp.w;
						let isdd = isda.magnitude_squared() * isdb.magnitude_squared();
						let isdp = isda.dot(isdb);
						(isdd.sqrt() * (v1 - (isdp * isdp) / isdd).max(v0).sqrt()) * (v1_2 * v1_2)
					} else {
						v0
					}
			} else {
				corner_area * v2
			}
		}
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Ssaa4<T> {
	parent: T,
}
impl<V: Real + MulAdd<Output = V>, T: Trace<Vec2<V>, V>> Trace<Vec2<V>, V> for Ssaa4<T> {
	type Cache = T::Cache;
	fn trace(&self, input: Vec2<V>, cache: &mut Self::Cache) -> V {
		let v1 = V::one();
		let v2 = v1 + v1;
		let v4 = v2 + v2;
		let v1_4 = v1 / v4;
		let vn1_4 = -v1_4;
		let scale = crate::SCALE.get();
		let scale_real: Mat2<V> = scale.map(|v| NumCast::from(v).unwrap());
		crate::SCALE.with(scale * Mat2::scaling_2d(0.5), || {
			Vec4::new(
				Vec2::new(vn1_4, vn1_4),
				Vec2::new(vn1_4, v1_4),
				Vec2::new(v1_4, v1_4),
				Vec2::new(vn1_4, vn1_4),
			)
			.map(|v| self.parent.trace(input + scale_real * v, cache))
			.sum() / v4
		})
	}
}

pub trait TraceExtSdf<V>: Trace<Vec2<V>, V> {
	/// Marching-square based anti-aliasing
	fn sdf2fac(self) -> Sdf2Fac<Self>
	where
		Self: Sized,
	{
		Sdf2Fac { parent: self }
	}
	/// Basic 4xSSAA
	fn ssaa4(self) -> Ssaa4<Self>
	where
		Self: Sized,
	{
		Ssaa4 { parent: self }
	}
}
impl<V: Real, T: Trace<Vec2<V>, V>> TraceExtSdf<V> for T {}
