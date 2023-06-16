#[cfg(feature = "color")]
pub mod color;

pub use vek;
pub mod prelude {
	#[cfg(feature = "color")]
	pub use crate::color::*;
	pub use vek::{
		self,
		mat::{Mat2, Mat3, Mat4},
		// we use our own geometry primitives
		//geom::{
		//	Aabb, Aabr, Disk, LineSegment2, LineSegment3, Ray, Rect, Rect3, Sphere,
		//	// these are named incorrectly in the library
		//	Ellipsis as Ellipse,
		//	Potato as Ellipsoid,
		//},
		num_traits::{
			real::Real, AsPrimitive, Bounded, CheckedAdd, CheckedDiv, CheckedEuclid, CheckedMul,
			CheckedNeg, CheckedRem, CheckedShl, CheckedShr, CheckedSub, Euclid, Float, FloatConst,
			FromPrimitive, Inv, MulAdd, MulAddAssign, Num, NumAssign, NumAssignOps, NumAssignRef,
			NumCast, NumOps, NumRef, One, Pow, PrimInt, RefNum, Saturating, SaturatingAdd,
			SaturatingMul, SaturatingSub, Signed, ToPrimitive, Unsigned, WrappingAdd, WrappingMul,
			WrappingNeg, WrappingShl, WrappingShr, WrappingSub, Zero,
		},
		ops::{
			Clamp, Clamp01, ClampMinus1, IsBetween, IsBetween01, Lerp, MulAdd as VekMulAdd, Slerp,
			Wrap,
		},
		quaternion::Quaternion,
		vec::{Extent2, Extent3, Vec2, Vec3, Vec4},
	};
}
