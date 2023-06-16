//+omnilive
// mode = "image"
use omnilive::prelude::*;

#[omnilive::main]
fn main() {
	//let circle1 = Circle(0.4)
	//	.sdf2fac()
	//	.map_out(|v| Color::RED * v as f32)
	//	.trans(Mat3::translation_2d(Vec2::new(0.45, 0.45)));
	//let circle2 = Circle(0.4)
	//	.sdf2fac()
	//	.map_out(|v| Color::BLUE * v as f32 / 4.0)
	//	.trans(Mat3::translation_2d(Vec2::new(0.55, 0.55)));
	//let background = Func(|pos: Vec2<f64>| Color::create(pos.x as f32, 1.0 - pos.y as f32, 0.0, 1.0));
	let func = Func(|pos: Vec2<f64>| (pos.x * 20.0).sin() + (pos.y * 20.0).tan()).sdf2fac();
	let background = Func(|_| Color::BLACK);
	omnitrace_live::image::present(
		Extent2::new(256, 256),
		background
			.then_blend_add(
				func.trans(Mat3::translation_2d(Vec2::new(0.5, 0.5)))
					.map_out(|v| Color::RED * v as f32),
			)
			.then_blend_add(
				func.trans(Mat3::translation_2d(Vec2::new(0.7, 0.7)))
					.map_out(|v| Color::GREEN * v as f32),
			)
			.then_blend_add(
				func.trans(Mat3::translation_2d(Vec2::new(0.3, 0.3)))
					.map_out(|v| Color::BLUE * v as f32),
			)
			.trans(
				Mat3::<f64>::translation_2d(Vec2::new(0.0, -1.0))
					* Mat3::<f64>::scaling_3d(Vec3::new(256.0, -256.0, 1.0)),
			)
			.map_in(|v: Vec2<u32>| v.map(|v| v as f64 + 0.5)),
	)
}
