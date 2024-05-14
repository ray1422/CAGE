use std::borrow::BorrowMut;

pub use bevy::math::cubic_splines::CubicCurve;
use bevy::math::Vec3;
type MyCubicCurve = CubicCurve<Vec3>;
fn concat(a: CubicCurve<Vec3>, b: CubicCurve<Vec3>) -> CubicCurve<Vec3> {
    let mut segs = a.segments();
}
