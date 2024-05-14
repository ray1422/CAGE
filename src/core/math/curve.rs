use bevy::math::{vec3, Vec3};

use self::quadratic::QuadraticBezierCurve;

pub mod quadratic;

pub struct Curve {
    curves: Vec<QuadraticBezierCurve>,
    // the prefix sum of the lengths of each curve
    sum_lengths: Vec<f32>,
}

impl Curve {
    pub fn length(&self) -> f32 {
        self.sum_lengths.last().unwrap().clone()
    }
    pub fn iter_positions(&self, n: isize) -> impl Iterator<Item = Vec3> + '_ {
        self.curves
            .iter()
            .flat_map(move |curve| curve.iter_positions(n))
    }
    pub fn offset(&self, right: f32, top: f32) -> Self {
        let mut rets = Vec::new();

        for curve in &self.curves {
            let curves = curve.to_curve().curves.iter().flat_map(|c|c.to_curve2(2., 1e2).curves).collect::<Vec<_>>();
            for curve in curves {
                let [p0, p1, p2] = curve.ctrl_pts;
                let v0 = p1 - p0;
                let v1 = p2 - p1;
                // calc normal
                let n0 = vec3(-v0.z, 0.0, v0.x).normalize();
                let n1 = vec3(-v1.z, 0.0, v1.x).normalize();
                let q0 = p0 + n0 * right;
                let q2 = p2 + n1 * right;
                // p1 is the intersection of the two normals
                // calculate the intersection of two lines
                // p0 + t * v0 = p2 + s * v1
                // t * v0 - s * v1 = p2 - p0
                let det = v0.x * v1.z - v0.z * v1.x;
                let q1 = if det.abs() < 1e-6 {
                    // parallel
                    println!("parallel");
                    (q2 + q0) / 2.0
                } else {
                    let t = (v1.z * (q2.x - q0.x) - v1.x * (q2.z - q0.z)) / det;
                    q0 + t * v0
                };

                rets.push(QuadraticBezierCurve::new([q0, q1 + top, q2]));
            }
            println!("{:?}", rets);
        }
        let sum_lengths = rets
            .iter()
            .map(|curve| curve.length())
            .scan(0.0, |sum, x| {
                *sum += x;
                Some(*sum)
            })
            .collect();
        Self {
            curves: rets,
            sum_lengths,
        }
    }
}
