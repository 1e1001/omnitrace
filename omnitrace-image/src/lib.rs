use std::fs::File;
use std::io;
use std::path::Path;

use omnitrace::prelude::*;
use omnitrace_math::prelude::*;
use palette::WithAlpha;

pub mod ext;
pub mod shapes;

pub mod prelude {
	pub use crate::ext::{TraceExtCompositing, TraceExtSdf, TraceExtVec2Transform};
	pub use crate::shapes::Circle;
	pub use omnitrace_math::prelude::*;
}

parameter_const::define! {
	/// World to Local
	pub copy_parameter SCALE: Mat2<f64> = Default::default();
}

// Replace with render(name, size, trace) -> Image + Image::save(self, path)
pub fn render<T: Trace<Vec2<u32>, Color>>(
	path: impl AsRef<Path>,
	size: Extent2<u32>,
	trace: T,
) -> io::Result<()> {
	let path = path.as_ref();
	let name = format!("Draw {path:?}");
	let total_len = size.w as u64 * size.h as u64;
	let iter = 0..total_len;
	let width = size.w as u64;
	let trace = trace
		.map_in(|v| Vec2::new((v % width) as u32, (v / width) as u32))
		.map_out(|v| {
			let (color, alpha): (_, f32) = v.split();
			palette::Srgb::from_linear(color).with_alpha((alpha * 255.0) as u8)
		});
	#[cfg(feature = "parallel")]
	let pixels = omnitrace::iterate_parallel_fast(name, iter, total_len, trace);
	#[cfg(not(feature = "parallel"))]
	let pixels = omnitrace::iterate_linear(name, iter, trace);
	let bar = omnitrace::pretty::bar::Bar::new(format!("Save {path:?}"), None);
	let pixels = palette::cast::into_component_slice(&pixels);
	let file = io::BufWriter::new(File::create(path)?);
	let mut encoder = png::Encoder::new(file, size.w, size.h);
	encoder.set_color(png::ColorType::Rgba);
	encoder.set_depth(png::BitDepth::Eight);
	encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455));
	encoder.write_header()?.write_image_data(&pixels)?;
	drop(bar);
	Ok(())
}
