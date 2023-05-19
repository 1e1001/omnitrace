pub use omnitrace_math::prelude::*;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Color {
	pub r: f32,
	pub g: f32,
	pub b: f32,
	pub a: f32,
}
macro_rules! base_color {
	($n:tt $r:tt $g:tt $b:tt $a:tt) => {
		#[doc = concat!("<div style=\"background-color:rgba(calc(100%*", $r, "),calc(100%*", $g, "),calc(100%*", $b, "),", $a, "); width: 100px; padding: 10px; border: 1px solid;\"></div>")]
		pub const $n: Self = Self::rgba($r, $g, $b, $a);
	};
}
impl Color {
	base_color!(NONE    0.0 0.0 0.0 0.0);
	base_color!(BLACK   0.0 0.0 0.0 1.0);
	base_color!(BLUE    0.0 0.0 1.0 1.0);
	base_color!(GREEN   0.0 1.0 0.0 1.0);
	base_color!(CYAN    0.0 1.0 1.0 1.0);
	base_color!(RED     1.0 0.0 0.0 1.0);
	base_color!(MAGENTA 1.0 0.0 1.0 1.0);
	base_color!(YELLOW  1.0 1.0 0.0 1.0);
	base_color!(WHITE   1.0 1.0 1.0 1.0);
	#[cfg_attr(feature = "inline", inline(always))]
	pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
		Self { r, g, b, a }
	}
	#[cfg_attr(feature = "inline", inline(always))]
	pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
		Self::rgba(r, g, b, 1.0)
	}
	#[cfg_attr(feature = "inline", inline(always))]
	pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
		Self::rgba(
			r as f32 / 255.0,
			g as f32 / 255.0,
			b as f32 / 255.0,
			a as f32 / 255.0,
		)
	}
	#[cfg_attr(feature = "inline", inline(always))]
	pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
		Self::rgba8(r, g, b, 255)
	}
	#[cfg_attr(feature = "inline", inline(always))]
	pub fn over(self, bottom: Self) -> Self {
		(Vec4::from(self) + Vec4::from(bottom) * (1.0 - self.a)).into()
	}
}
impl From<Vec4<f32>> for Color {
	fn from(v: Vec4<f32>) -> Self {
		Self::rgba(v.x, v.y, v.z, v.w)
	}
}
impl From<Color> for Vec4<f32> {
	fn from(v: Color) -> Self {
		Self::new(v.r, v.g, v.b, v.a)
	}
}
