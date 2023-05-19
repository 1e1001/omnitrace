pub use palette;
/// straight linear sRGBA
pub type Color = palette::Alpha<palette::LinSrgb<f32>, f32>;
macro_rules! impl_color_utils {
	(color_trait $n:tt $r:tt $g:tt $b:tt $a:tt) => {
		#[doc = concat!("<div style=\"background-color:rgba(calc(100%*", $r, "),calc(100%*", $g, "),calc(100%*", $b, "),", $a, "); width: 100px; padding: 10px; border: 1px solid;\"></div> rgba(", $r, ", ", $g, ", ", $b, ", ", $a, ")")]
		const $n: Self;
	};
	(color_impl $n:tt $r:tt $g:tt $b:tt $a:tt) => {
		const $n: Self = color_new_const($r, $g, $b, $a);
	};
	($([$c0:tt $c1:tt $c2:tt $c3:tt $c4:tt])*) => {
		pub trait ColorUtils {
			fn new(r: f32, g: f32, b: f32, a: f32) -> Self;
			$(impl_color_utils!(color_trait $c0 $c1 $c2 $c3 $c4);)*
		}
		impl ColorUtils for Color {
			fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
				color_new_const(r, g, b, a)
			}
			$(impl_color_utils!(color_impl $c0 $c1 $c2 $c3 $c4);)*
		}
	};
}
pub const fn color_new_const(r: f32, g: f32, b: f32, a: f32) -> Color {
	Color {
		color: palette::LinSrgb::new(r, g, b),
		alpha: a,
	}
}
impl_color_utils! {
	[NONE    0.0 0.0 0.0 0.0]
	[BLACK   0.0 0.0 0.0 1.0]
	[BLUE    0.0 0.0 1.0 1.0]
	[GREEN   0.0 1.0 0.0 1.0]
	[CYAN    0.0 1.0 1.0 1.0]
	[RED     1.0 0.0 0.0 1.0]
	[MAGENTA 1.0 0.0 1.0 1.0]
	[YELLOW  1.0 1.0 0.0 1.0]
	[WHITE   1.0 1.0 1.0 1.0]
}
// macro_rules! def_base_color {
// 	($n:tt $r:tt $g:tt $b:tt $a:tt) => {
// 		#[doc = concat!("<div style=\"background-color:rgba(calc(100%*", $r, "),calc(100%*", $g, "),calc(100%*", $b, "),", $a, "); width: 100px; padding: 10px; border: 1px solid;\"></div> rgba(", $r, ", ", $g, ", ", $b, ", ", $a, ")")]
// 		pub const $n: Color = create($r, $g, $b, $a);
// 	};
// }
// pub mod base_color {
// 	use super::Color;
// 	pub const fn create(r: f32, g: f32, b: f32, a: f32) -> Color {
// 		Color {
// 			color: palette::LinSrgb::new(r, g, b),
// 			alpha: a,
// 		}
// 	}
// 	def_base_color!(NONE    0.0 0.0 0.0 0.0);
// 	def_base_color!(BLACK   0.0 0.0 0.0 1.0);
// 	def_base_color!(BLUE    0.0 0.0 1.0 1.0);
// 	def_base_color!(GREEN   0.0 1.0 0.0 1.0);
// 	def_base_color!(CYAN    0.0 1.0 1.0 1.0);
// 	def_base_color!(RED     1.0 0.0 0.0 1.0);
// 	def_base_color!(MAGENTA 1.0 0.0 1.0 1.0);
// 	def_base_color!(YELLOW  1.0 1.0 0.0 1.0);
// 	def_base_color!(WHITE   1.0 1.0 1.0 1.0);
// }
pub use palette::blend::{Blend, BlendWith, Compose};
