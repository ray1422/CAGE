use bevy::math::{
    cubic_splines::{CubicBezier, CubicCurve, CubicGenerator},
    vec2, vec3, Vec2, Vec3,
};
use peroxide::fuga::*;

/// CageCurve wraps bevy's curve and add the important missing functions
/// for our use case.
/// Usage:
/// ```
/// use bevy::math::Vec3;
/// use cage::core::math::curve::*;
/// let u = CageBezierCurve::new([
///     Vec3::new(0.0, 0.0, 0.0),
///     Vec3::new(0.3, 0.3, 0.0),
///     Vec3::new(0.8, 0.8, 0.0),
///     Vec3::new(1.0, 1.0, 0.0),
/// ]);
/// let l = u.length();
/// println!("{}", l);
///
/// ```
pub trait CageCurve {
    /// length of arc
    fn length(&self) -> f32;

    /// sample the position on curve. t is in [0, 1]
    fn position(&self, t: f32) -> Vec3;

    /// is_intersect with another curve
    fn has_intersection(&self, other: &impl CageCurve) -> bool {
        // FIXME: this is the simple way to detect by tangent line.
        // we need to optimized it someday.
        self.iter_positions(4)
            .collect::<Vec<Vec3>>()
            .windows(2)
            .any(|p| {
                let p0 = p[0]; // p0.x, p0.y, p0.z
                let p1 = p[1];
                other
                    .iter_positions(4)
                    .collect::<Vec<Vec3>>()
                    .windows(2)
                    .any(|q| {
                        let q0 = q[0];
                        let q1 = q[1];
                        match find_intersection_xz(p0, p1, q0, q1) {
                            Some((t, s)) => t >= 0. && t <= 1. && s >= 0. && s <= 1.,
                            None => false,
                        }
                    })
            })
    }

    /// iter the positions on the curve
    fn iter_positions(&self, n: usize) -> impl Iterator<Item = Vec3> + '_;

    // shift the curve to normal direction by n. the new length will be different.
    fn shift(&self, n: Vec2) -> impl CageCurve;
}

// find_intersection of two lines constructed by p0 to p1 and q0 to q1.
fn find_intersection(p0: Vec3, p1: Vec3, q0: Vec3, q1: Vec3) -> Option<(f32, f32)> {
    let u = p1 - p0;
    let v = q1 - q0;
    let mut mat: Vec<f64> = vec![];
    for i in 0..3 {
        mat.push(-u[i] as f64);
        mat.push(v[i] as f64);
        mat.push((p0[i] - q0[i]) as f64);
    }
    let mat = Matrix {
        data: mat,
        row: 3,
        col: 3,
        shape: Row,
    };
    let mat = mat.rref();
    println!("{:?}", mat);
    let no_sol = (0..3)
        .any(|i| mat[(i, 0)].abs() < 1e-6 && mat[(i, 1)].abs() < 1e-6 && mat[(i, 2)].abs() > 1e-6);
    if no_sol || mat[(0, 1)].abs() > 1e-6 || mat[(1, 0)].abs() > 1e-6 {
        println!("no or inf sols");

        if !no_sol {
            println!("inf sol");
            for i in 0..2 {
                if mat[(i, 0)].abs() < 1e-6 || mat[(i, 1)].abs() < 1e-6 {
                    continue;
                }
                let a = mat[(i, 0)];
                let b = mat[(i, 1)];
                let n = mat[(i, 2)];
                // take t = 0.01
                println!("take t = 0.01");
                if mat[(i, 1)] > 1e-6 {
                    let s = (-(n + 0.01 * a) / b) as f32;
                    if 0.0 <= s && s <= 1.0 {
                        return Some((0.01, s));
                    }
                }
                // take t = 0.99
                println!("take t = 0.99");
                if mat[(i, 1)] > 1e-6 {
                    let s = ((-n - 0.99 * a) / b) as f32;
                    if 0.0 <= s && s <= 1.0 {
                        return Some((0.99, s));
                    }
                }

                // take s = 0.01
                println!("take s = 0.01");
                if mat[(i, 0)] > 1e-6 {
                    let t = -((-n - b * 0.01) / a) as f32;
                    if 0.0 <= t && t <= 1.0 {
                        return Some((t, 0.01));
                    }
                }

                // take s = 0.99
                println!("take s = 0.99");
                if mat[(i, 0)] > 1e-6 {
                    let t = -((-n - 0.99 * b) / a) as f32;
                    if 0.0 <= t && t <= 1.0 {
                        return Some((t, 0.99));
                    }
                }
            }
        }
        None
    } else {
        let t = mat[(0, 2)] as f32;
        let s = mat[(1, 2)] as f32;
        Some((t, s))
    }
}

/// find_intersection_xz finds the intersection of two lines in x-z plane, and check
/// if z axis is close enough.
fn find_intersection_xz(p0_v3: Vec3, p1_v3: Vec3, q0_v3: Vec3, q1_v3: Vec3) -> Option<(f32, f32)> {
    let p0 = vec2(p0_v3.x, p0_v3.z);
    let p1 = vec2(p1_v3.x, p1_v3.z);
    let q0 = vec2(q0_v3.x, q0_v3.z);
    let q1 = vec2(q1_v3.x, q1_v3.z);
    let u = p1 - p0;
    let v = q1 - q0;
    let mut mat: Vec<f64> = vec![];
    for i in 0..2 {
        mat.push(-u[i] as f64);
        mat.push(v[i] as f64);
        mat.push((p0[i] - q0[i]) as f64);
    }
    let mat = Matrix {
        data: mat,
        row: 2,
        col: 3,
        shape: Row,
    };
    let mat = mat.rref();
    println!("{:?}", mat);
    let no_sol = (0..2)
        .any(|i| mat[(i, 0)].abs() < 1e-6 && mat[(i, 1)].abs() < 1e-6 && mat[(i, 2)].abs() > 1e-6);
    if no_sol || mat[(0, 1)].abs() > 1e-6 || mat[(1, 0)].abs() > 1e-6 {
        println!("no or inf sols");
        if !no_sol {
            // if inf sol, fallback to find_intersection
            return find_intersection(p0_v3, p1_v3, q0_v3, q1_v3);
        }
        None
    } else {
        const VERTICAL_EPS: f32 = 1e-1;
        let t = mat[(0, 2)] as f32;
        let s = mat[(1, 2)] as f32;
        let p = p0_v3 + (p1_v3 - p0_v3) * t;
        let q = q0_v3 + (q1_v3 - q0_v3) * s;
        if (p - q).length() < VERTICAL_EPS {
            Some((t, s))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct CageBezierCurve {
    #[allow(dead_code)]
    pub control_points: [Vec3; 4],
    bevy_curve: CubicCurve<Vec3>,
    _length: f32,
}

impl CageBezierCurve {
    pub fn new(control_points: [Vec3; 4]) -> Self {
        let curve = CubicBezier::new([control_points]).to_curve();
        // iter to get the length
        let mut length = 0.0;
        let mut last = curve.position(0.0);
        for (i, p) in curve.iter_positions(256).enumerate() {
            if i > 0 {
                length += (p - last).length();
            }
            last = p;
        }

        CageBezierCurve {
            control_points: control_points,
            bevy_curve: curve,
            _length: length,
        }
    }
    fn shift(&self, n: Vec2) -> CageShiftCurve<CageBezierCurve> {
        CageShiftCurve::<CageBezierCurve>::new(self.clone(), n)
    }
}

impl CageCurve for CageBezierCurve {
    fn length(&self) -> f32 {
        self._length
    }

    fn position(&self, t: f32) -> Vec3 {
        self.bevy_curve.position(t)
    }

    fn iter_positions(&self, n: usize) -> impl Iterator<Item = Vec3> + '_ {
        self.bevy_curve.iter_positions(n)
    }

    fn shift(&self, n: Vec2) -> impl CageCurve {
        self.shift(n)
    }
}

#[derive(Debug, Clone)]
pub struct CageShiftCurve<T>
where
    T: CageCurve + Clone,
{
    curve: T,
    shift: Vec2,
}

impl<T> CageShiftCurve<T>
where
    T: CageCurve + Clone,
{
    pub fn new(curve: T, shift: Vec2) -> Self {
        CageShiftCurve {
            curve: curve,
            shift: shift,
        }
    }

    pub fn shift(&self, shift: Vec2) -> CageShiftCurve<T> {
        CageShiftCurve::<T>::new(self.curve.clone(), self.shift + shift)
    }

    pub fn iter_positions(&self, n: usize) -> impl Iterator<Item = Vec3> + '_ {
        // iter over the positions, shift the position by n of normal direction,
        // then construct new curve by the shifted positions.
        self.curve
            .iter_positions(n)
            .collect::<Vec<Vec3>>()
            .windows(2)
            .collect::<Vec<_>>()
            .iter()
            .map(|p| {
                let p0 = p[0];
                let p1 = p[1];
                let u = p1 - p0;
                // get normal vector on x-z plane
                let normal = (Vec3::new(-u.z, 0.0, u.x).normalize() + Vec3::new(0.0, 1.0, 0.0))
                    * vec3(
                        self.shift.x as f32,
                        self.shift.y as f32,
                        self.shift.x as f32,
                    );
                p0 + normal
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}
impl<T> CageCurve for CageShiftCurve<T>
where
    T: CageCurve + Clone,
{
    fn length(&self) -> f32 {
        self.curve.length()
    }

    fn position(&self, t: f32) -> Vec3 {
        self.curve.position(t)
    }

    fn iter_positions(&self, n: usize) -> impl Iterator<Item = Vec3> + '_ {
        self.iter_positions(n)
    }

    fn shift(&self, n: Vec2) -> impl CageCurve {
        self.shift(n)
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::{vec2, vec3, Vec3};
    #[test]
    fn test_intersection_xz() {
        let p0 = vec3(0., 0., 1.);
        let p1 = vec3(1., 0., 0.);
        let q0 = vec3(0., 0., 0.);
        let q1 = vec3(1., 0., 1.);
        let ret = find_intersection_xz(p0, p1, q0, q1);
        println!("{:?}", ret);
        assert!(ret.is_some())
    }

    #[test]
    fn test_intersection_xz_vertical() {
        let p0 = vec3(0., 1., 1.);
        let p1 = vec3(1., 1., 0.);
        let q0 = vec3(0., 0., 0.);
        let q1 = vec3(1., 0., 1.);
        let ret = find_intersection_xz(p0, p1, q0, q1);
        println!("{:?}", ret);
        assert!(ret.is_none())
    }

    #[test]
    fn test_intersection_same() {
        let p0 = vec3(0., 0., 1.);
        let p1 = vec3(1., 0., 0.);
        let ret = find_intersection(p0, p1, p0, p1);
        println!("{:?}", ret);
        assert!(ret.is_none());
    }

    #[test]
    fn test_intersection_zero() {
        let p0 = Vec3::ZERO;
        let p1 = Vec3::ZERO;
        let ret = find_intersection(p0, p1, p0, p1);
        println!("{:?}", ret);
        // undefined result, just ensure not panic
    }

    #[test]
    fn test_intersection_xz_zero() {
        let p0 = Vec3::ZERO;
        let p1 = Vec3::ZERO;
        let ret = find_intersection_xz(p0, p1, p0, p1);
        println!("{:?}", ret);
        // undefined result, just ensure not panic
    }

    #[test]
    fn test_intersection_parallel() {
        let p0 = vec3(0., 0., 2.);
        let p1 = vec3(2., 0., 4.);
        let q0 = vec3(0., 0., 4.);
        let q1 = vec3(2., 0., 6.);
        let ret = find_intersection(p0, p1, q0, q1);
        println!("{:?}", ret);
        assert!(ret.is_none());
    }

    #[test]
    fn test_intersection_xz_parallel() {
        let p0 = vec3(0., 0., 2.);
        let p1 = vec3(2., 0., 4.);
        let q0 = vec3(0., 0., 4.);
        let q1 = vec3(2., 0., 6.);
        let ret = find_intersection(p0, p1, q0, q1);
        println!("{:?}", ret);
        assert!(ret.is_none());
    }

    use crate::core::math::curve::{find_intersection_xz, CageBezierCurve, CageCurve};

    use super::find_intersection;

    #[test]
    fn test_curve_intersection() {
        use super::*;
        let curve1 = CageBezierCurve::new([
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.3, 0.0, 0.3),
            Vec3::new(0.8, 0.0, 0.8),
            Vec3::new(1.0, 0.0, 1.0),
        ]);
        let curve2 = CageBezierCurve::new([
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.8, 0.0, 0.3),
            Vec3::new(0.3, 0.0, 0.8),
            Vec3::new(0.0, 0.0, 1.0),
        ]);
        assert!(curve1.has_intersection(&curve2));

        let curve3 = CageBezierCurve::new([
            Vec3::new(10.0, 0.0, 10.0),
            Vec3::new(11.0, 0.0, 11.0),
            Vec3::new(12.0, 0.0, 12.0),
            Vec3::new(20.0, 0.0, 20.0),
        ]);
        assert!(!curve1.has_intersection(&curve3));
    }

    #[test]
    fn test_curve_intersection_parallel() {
        let curve1 = CageBezierCurve::new([
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
        ]);

        let curve2 = CageBezierCurve::new([
            Vec3::new(1.0, 10.1, 0.0),
            Vec3::new(2.0, 10.1, 0.0),
            Vec3::new(3.0, 10.1, 0.0),
            Vec3::new(4.0, 10.1, 0.0),
        ]);

        let curve3 = CageBezierCurve::new([
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
        ]);

        let curve4 = CageBezierCurve::new([
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
            Vec3::new(5.0, 5.0, 0.0),
        ]);

        assert!(!curve1.has_intersection(&curve2));
        assert!(!curve2.has_intersection(&curve1));
        // same line
        assert!(curve1.has_intersection(&curve3));

        // same line, different segment
        assert!(curve1.has_intersection(&curve4));
    }

    #[test]
    fn test_curve_edge_cases() {
        let curve = CageBezierCurve::new([
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ]);
        let curve2 = CageBezierCurve::new([
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
        ]);
        assert!(curve.length() < 1e-6);
        assert!(!curve.has_intersection(&curve2));
    }

    #[test]
    fn test_curve_length() {
        use super::*;
        let curve = CageBezierCurve::new([
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.65, 0.0),
            Vec3::new(0.35, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
        ]);
        assert!((curve.length() - 1.61).abs() < 1e-2, "{}", curve.length());
    }
    #[test]
    fn test_curve_shift() {
        let curve = CageBezierCurve::new([
            Vec3::new(-4.0, 0.0, -4.0),
            Vec3::new(-2.0, 0.0, 4.0),
            Vec3::new(2.0, 0.0, -4.0),
            Vec3::new(4.0, 0.0, 4.0),
        ]);
        let new_curve = curve.shift(vec2(1.0, 0.0));
        println!("{:?}", new_curve);
        assert!(!new_curve.has_intersection(&curve));
    }

    #[test]
    fn test_zero_curve_shift() {
        let curve = CageBezierCurve::new([Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO])
            .shift(vec2(1.0, 0.));
        assert!((curve.length() - 0.0).abs() < 1e-6);
    }
}
