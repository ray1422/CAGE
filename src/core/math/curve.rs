use std::cmp::min;

use bevy::math::{vec3, Vec3};

use self::quadratic::QuadraticBezierCurve;

pub mod quadratic;

#[derive(Debug, Clone)]
pub struct Curve {
    curves: Vec<QuadraticBezierCurve>,
    // the prefix sum of the lengths of each curve
    sum_lengths: Vec<f32>,
}

impl Curve {
    pub fn from_curves(curves: Vec<QuadraticBezierCurve>) -> Self {
        let sum_lengths = curves
            .iter()
            .map(|curve| curve.length())
            .scan(0.0, |sum, x| {
                *sum += x;
                Some(*sum)
            })
            .collect();
        Self {
            curves,
            sum_lengths,
        }
    }
    pub fn slice(&self, start: f32, end: f32) -> Self {
        let start_pt = self.position(start);
        let end_pt = self.position(end);
        self.split_at(start_pt).1.split_at(end_pt).0
    }

    pub fn slice_by_length(&self, start: f32, end: f32) -> Result<Self, ()> {
        let start = start / self.length();
        let end = end / self.length();
        if start >= end {
            return Err(());
        }
        Ok(self.slice(start, end))
    }

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

    pub fn start(&self) -> Vec3 {
        self.curves[0].start()
    }

    pub fn end(&self) -> Vec3 {
        self.curves.last().unwrap().end()
    }

    pub fn distance_to(&self, pt: Vec3) -> f32 {
        self.curves
            .iter()
            .map(|curve| curve.distance_to(pt))
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(f32::MAX)
    }

    // two new curve and the t value
    pub fn split_at(&self, pt: Vec3) -> (Self, Self) {
        // determine the curve to split
        let mut idx: usize = 0;
        let mut min_dist = f32::MAX;
        for (i, curve) in self.curves.iter().enumerate() {
            let dist = curve.distance_to(pt);
            if dist < min_dist {
                min_dist = dist;
                idx = i;
            }
        }

        let (mut curve1, curve2) = self.curves[idx].split_at(pt);
        let mut curves_1 = self.curves[..idx].to_vec();
        curves_1.append(&mut curve1.curves);
        let mut curves_2 = curve2.curves;
        curves_2.extend_from_slice(&self.curves[idx + 1..]);
        (Self::from_curves(curves_1), Self::from_curves(curves_2))
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
        let idx = self
            .sum_lengths
            .binary_search_by(|&sum_length| {
                sum_length
                    .partial_cmp(&l)
                    .unwrap_or(std::cmp::Ordering::Less)
            })
            .unwrap_or_else(|idx| min(idx, self.sum_lengths.len() - 1));
        let prefix_len = if idx == 0 {
            0.
        } else {
            self.sum_lengths[idx - 1]
        };
        // iter over length_of curve
        let mut low = 0;
        let mut high = 1024;
        while low < high {
            let mid = (low + high) / 2;
            let t = mid as f32 / 1024.0;
            let len = self.curves[idx].length_of(t);
            if len < l - prefix_len {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        let t = low as f32 / 1024.0;
        return self.curves[idx].position(t);
        // binary search was code by GPT, here's the original code:
        // iter over length_of curve
        // for t in 0..1000 {
        //     let t = t as f32 / 1000.0;
        //     let len = self.curves[idx].length_of(t);
        //     if len >= l - prefix_len {
        //         return self.curves[idx].position(t);
        //     }
        // }
        // let t = (l - prefix_len) / (self.sum_lengths[idx] - prefix_len);
        // self.curves[idx].position(t)
    }
    pub fn velocity(&self, t: f32) -> Vec3 {
        const EPS: f32 = 1e-6;
        // TODO: improve the accuracy by calculate the actual
        // derivative instead of numerical solution
        if t < EPS {
            return self.curves[0].position(1e-6) - self.curves[0].position(0.0);
        }
        if t > 1.0 - 1e-6 {
            return self.curves.last().unwrap().position(1.0)
                - self.curves.last().unwrap().position(1.0 - EPS);
        }
        return self.position(t + EPS) - self.position(t - EPS);
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
