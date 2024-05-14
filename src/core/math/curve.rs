use bevy::{
    ecs::query::Or,
    math::{vec3, Vec3},
};

use self::quadratic::QuadraticBezierCurve;

pub mod quadratic;

pub struct Curve {
    curves: Vec<QuadraticBezierCurve>,
    // the prefix sum of the lengths of each curve
    sum_lengths: Vec<f32>,
}

impl Curve {
    pub fn form_two_velocity(p: Vec3, v: Vec3, q: Vec3, u: Vec3) -> Self {
        let len = (q - p).length();
        let p0 = p;
        let p1 = p + v.normalize() * len / 3.0;
        let p2 = q - u.normalize() * len / 3.0;
        let p3 = q;
        Self::from_4_points(p0, p1, p2, p3)
    }

    // construct a curve that smoothly connects the two curves from 4 points
    pub fn from_4_points(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        let mid = (p1 + p2) / 2.0;
        let curve1 = QuadraticBezierCurve::new([p0, p1, mid]);
        let curve2 = QuadraticBezierCurve::new([mid, p2, p3]);
        Self {
            sum_lengths: vec![curve1.length(), curve1.length() + curve2.length()],
            curves: vec![curve1, curve2],
        }
    }

    pub fn length(&self) -> f32 {
        self.sum_lengths.last().unwrap().clone()
    }
    pub fn iter_positions(&self, n: isize) -> impl Iterator<Item = Vec3> + '_ {
        self.curves
            .iter()
            .flat_map(move |curve| curve.iter_positions(n))
    }

    // get the position of the curve at t. t is in (0, 1)
    pub fn position(&self, t: f32) -> Vec3 {
        let l = t * self.length();
        let mut idx = self.curves.len() - 1;
        for (i, &sum_length) in self.sum_lengths.iter().enumerate() {
            if sum_length >= l {
                idx = i;
                break;
            }
        }
        let prefix_len = if idx == 0 {
            0.
        } else {
            self.sum_lengths[idx - 1]
        };
        let t = (l - prefix_len) / (self.sum_lengths[idx] - prefix_len);
        self.curves[idx].position(t)
    }
    pub fn velocity(&self, t: f32) -> Vec3 {
        // TODO: Optimize this
        if t < 1e-6 {
            return self.curves[0].position(1e-6) - self.curves[0].position(0.0);
        }
        if t > 1.0 - 1e-6 {
            return self.curves.last().unwrap().position(1.0)
                - self.curves.last().unwrap().position(1.0 - 1e-6);
        }
        return self.position(t + 1e-6) - self.position(t - 1e-6);
    }

    pub fn offset(&self, right: f32, top: f32) -> Self {
        let mut rets = Vec::new();

        for curve in &self.curves {
            let curves = curve
                .to_curve()
                .curves
                .iter()
                .flat_map(|c| c.to_curve2(2., 1e2).curves)
                .collect::<Vec<_>>();
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
