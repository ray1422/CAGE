use bevy::math::Vec3;

#[derive(Clone, Debug)]
pub struct QuadraticBezierCurve {
    pub ctrl_pts: [Vec3; 3],
}
impl QuadraticBezierCurve {
    pub fn new(ctrl_pts: [Vec3; 3]) -> Self {
        Self { ctrl_pts }
    }
    pub fn to_curve(&self) -> super::Curve {
        super::Curve {
            curves: vec![self.clone()],
            sum_lengths: vec![self.length()],
        }
    }
    pub fn to_curve2(&self, min_angle: f32, max_length: f32) -> super::Curve {
        // subdivide the curve into multiple segments
        // that each segment has an angle of control points less than min_angle
        let mut curves = Vec::new();
        let (p0, p1, p2) = (self.ctrl_pts[0], self.ctrl_pts[1], self.ctrl_pts[2]);
        if (p1 - p0).length() < 1e-4 || (p1 - p2).length() < 1e-4 {
            return super::Curve {
                curves: vec![self.clone()],
                sum_lengths: vec![self.length()],
            };
        }
        // check the angle between the two tangents
        let v = p1 - p0;
        let u = p2 - p1;
        let angle = v.angle_between(-u);
        println!("angle: {:?}", angle);
        if angle > min_angle && self.length() < max_length {
            return super::Curve {
                curves: vec![self.clone()],
                sum_lengths: vec![self.length()],
            };
        }
        let left_mid = (p0 + p1) / 2.0;
        let right_mid = (p1 + p2) / 2.0;
        let mid = (left_mid + right_mid) / 2.0;
        let left_curve = QuadraticBezierCurve::new([p0, left_mid, mid]);
        let right_curve = QuadraticBezierCurve::new([mid, right_mid, p2]);
        curves.append(&mut left_curve.to_curve2(min_angle, max_length).curves);
        curves.append(&mut right_curve.to_curve2(min_angle, max_length).curves);
        // sum_lengths is prefix sum of the lengths of each curve
        let sum_lengths: Vec<f32> = curves.iter().map(|c| c.length()).collect();
        // calculate the prefix sum of the lengths of each curve
        let sum_lengths = sum_lengths
            .iter()
            .scan(0.0, |acc, &x| {
                *acc += x;
                Some(*acc)
            })
            .collect();

        super::Curve {
            curves,
            sum_lengths,
        }
    }

    // construct parallel curve
    pub fn iter_positions(&self, n: isize) -> impl Iterator<Item = Vec3> + '_ {
        let mut t = 0.0;
        let step = 1.0 / n as f32;
        std::iter::from_fn(move || {
            if t > 1.0 {
                return None;
            }
            let pos = self.position(t);
            t += step;
            Some(pos)
        })
    }

    pub fn position(&self, t: f32) -> Vec3 {
        // Bezier curve
        // B(t) = (1-t)^2 * P0 + 2(1-t)t * P1 + t^2 * P2
        let one_minus_t = 1.0 - t;
        let one_minus_t_sq = one_minus_t * one_minus_t;
        let t_sq = t * t;
        let p0 = self.ctrl_pts[0];
        let p1 = self.ctrl_pts[1];
        let p2 = self.ctrl_pts[2];
        one_minus_t_sq * p0 + 2.0 * one_minus_t * t * p1 + t_sq * p2
    }

    // we ignore z axis for now
    pub fn length(&self) -> f32 {
        self.length_of(1.)
    }

    pub fn length_of(&self, t: f32) -> f32 {
        const EPS: f32 = 1e-4;
        // https://members.loria.fr/SHornus/quadratic-arc-length.html
        // https://stackoverflow.com/questions/11854907/calculate-the-length-of-a-segment-of-a-quadratic-bezier
        // the following is the code snippet from the above link
        /*
        ```c++
        float x0,x1,x2,y0,y1,y2;      // control points of Bezier curve
        float get_l_analytic(float t) // get arc length from parameter t=<0,1>
        {
            float ax,ay,bx,by,A,B,C,b,c,u,k,L;
            ax=x0-x1-x1+x2;
            ay=y0-y1-y1+y2;
            bx=x1+x1-x0-x0;
            by=y1+y1-y0-y0;
            A=4.0*((ax*ax)+(ay*ay));
            B=4.0*((ax*bx)+(ay*by));
            C=     (bx*bx)+(by*by);
            b=B/(2.0*A);
            c=C/A;
            u=t+b;
            k=c-(b*b);
            L=0.5*sqrt(A)*
                (
                (u*sqrt((u*u)+k))
                -(b*sqrt((b*b)+k))
                +(k*log(fabs((u+sqrt((u*u)+k))/(b+sqrt((b*b)+k)))))
                );
            return L;
        }
        ```
         */
        let [p0, p1, p2] = self.ctrl_pts;

        // if p1 is close to p0 or p2, we can approximate the curve with a line
        if (p1 - p0).length() < EPS || (p1 - p2).length() < EPS {
            return (p2 - p0).length() * t;
        }
        // (p0 - p1) - (p1 - p2): difference between the two tangents
        let p_a = p0 - p1 - p1 + p2;
        let p_b = p1 + p1 - p0 - p0;
        let cap_a = 4.0 * p_a.length_squared();
        let cap_b = 4.0 * p_a.dot(p_b);

        // FIXME: we need to handle the case when cap_a is close to 0
        if cap_a < EPS {
            return (p2 - p0).length() * t;
        }
        let cap_c = p_b.length_squared();

        let b = cap_b / ((2.0 * cap_a) + EPS);
        let c = cap_c / (cap_a + EPS);
        let u = t + b;
        let k = c - (b * b);
        let ret = 0.5
            * (cap_a).max(0.).sqrt()
            * ((u * ((u * u) + k).max(0.).sqrt()) - (b * ((b * b) + k).max(0.).sqrt())
                + (k * (((u + ((u * u) + k).max(0.).sqrt())
                    / (b + ((b * b) + k).max(0.).sqrt() + EPS))
                    .abs()
                    + EPS)
                    .max(EPS)
                    .ln()));
        if ret.is_nan() {
            println!(
                "nan!!! p0: {:?}, p1: {:?}, p2: {:?}, t: {:?}",
                p0, p1, p2, t
            );
            // TODO: report this error
            0.
        } else {
            ret
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_quadratic_curve() {
        let curve = super::QuadraticBezierCurve::new([
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        ]);
        let pos = curve.position(0.5);
        assert_eq!(pos, Vec3::new(1.0, 0.5, 0.0));
    }

    #[test]
    fn test_quadratic_curve_length_edge_case() {
        let curve = super::QuadraticBezierCurve::new([
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ]);
        let len = curve.length();
        assert!(len < 1e-2, "curve length: {:?}", len);

        let curve = super::QuadraticBezierCurve::new([
            Vec3::new(-5.0, -5.0, -5.0),
            Vec3::new(-5.0, -5.0, -4.0),
            Vec3::new(-5.0, -5.0, -3.0),
        ]);
        let len = curve.length();
        let real_len = (Vec3::new(-5.0, -5.0, -5.0) - Vec3::new(-5.0, -5.0, -3.0)).length();
        assert!(
            (len - real_len).abs() < 1e-2,
            "curve length: {:?}, real length: {:?}",
            len,
            real_len,
        );

        // Vec3(-5.0, -5.0, -5.0), p1: Vec3(-5.0, -5.0, -4.0), p2: Vec3(-5.0, -5.0, -5.0)
        let curve = super::QuadraticBezierCurve::new([
            Vec3::new(-5.0, -5.0, -5.0),
            Vec3::new(-5.0, -5.0, -4.0),
            Vec3::new(-5.0, -5.0, -5.0),
        ]);
        let len = curve.length();
        let real_len = curve
            .iter_positions(128)
            .collect::<Vec<Vec3>>()
            .windows(2)
            .fold(0.0, |acc, p| acc + (p[0] - p[1]).length());
        assert!(
            (len - real_len).abs() < 1e-2,
            "curve length: {:?}, real length: {:?}",
            len,
            real_len,
        );

        let curve = super::QuadraticBezierCurve::new([
            Vec3::new(-5.0, -5.0, -5.0),
            Vec3::new(-5.0, -5.0, -4.0),
            Vec3::new(-5.0, -5.0, -6.0),
        ]);
        let len = curve.length();
        let real_len = curve
            .iter_positions(128)
            .collect::<Vec<Vec3>>()
            .windows(2)
            .fold(0.0, |acc, p| acc + (p[0] - p[1]).length());
        assert!(
            (len - real_len).abs() < 1e-2,
            "curve length: {:?}, real length: {:?}",
            len,
            real_len,
        );

        let curve = super::QuadraticBezierCurve::new([
            Vec3::new(0., 2., 0.),
            Vec3::new(0., 2., 0.),
            Vec3::new(0., -5.0, 0.),
        ]);
        let len = curve.length();
        let real_len = curve
            .iter_positions(128)
            .collect::<Vec<Vec3>>()
            .windows(2)
            .fold(0.0, |acc, p| acc + (p[0] - p[1]).length());
        assert!(
            (len - real_len).abs() < 1e-2,
            "curve length: {:?}, real length: {:?}",
            len,
            real_len,
        );

        let curve = super::QuadraticBezierCurve::new([
            Vec3::new(0., 0.0, 0.),
            Vec3::new(0., 2.2, 0.),
            Vec3::new(0., 4.0, 0.),
        ]);
        let len = curve.length();
        let real_len = curve
            .iter_positions(128)
            .collect::<Vec<Vec3>>()
            .windows(2)
            .fold(0.0, |acc, p| acc + (p[0] - p[1]).length());
        assert!(
            (len - real_len).abs() < 1e-2,
            "curve length: {:?}, real length: {:?}",
            len,
            real_len,
        );
    }

    #[test]
    #[ignore]
    // enable optimization for this test case
    fn test_quadratic_all_curve_length() {
        let mut points = Vec::new();
        for i in -5..5 {
            for j in -5..5 {
                for k in -5..5 {
                    points.push(Vec3::new(i as f32, j as f32, k as f32));
                }
            }
        }

        for i in 0..points.len() {
            for j in 0..points.len() {
                for k in 0..points.len() {
                    let index = [points[i].clone(), points[j].clone(), points[k].clone()];

                    let curve = super::QuadraticBezierCurve::new(index);
                    let len = curve.length();
                    let real_len = curve
                        .iter_positions(5)
                        .collect::<Vec<Vec3>>()
                        .windows(2)
                        .fold(0.0, |acc, p| acc + (p[0] - p[1]).length());
                    if (len - real_len).abs() > 9e-1 {
                        println!(
                            "curve length: {:?}, real length: {:?}. test case: {:?} {:?} {:?}",
                            len, real_len, index[0], index[1], index[2]
                        );
                        panic!("test failed");
                    }
                    assert!(
                        (len - real_len).abs() < 9e-1,
                        "curve length: {:?}, real length: {:?}. test case: {:?} {:?} {:?}",
                        len,
                        real_len,
                        index[0],
                        index[1],
                        index[2]
                    );
                }
            }
        }
    }
    #[test]
    fn test_divide_with_angle() {
        let curves = [
        // construct a extremely sharp curve
            super::QuadraticBezierCurve::new([
                Vec3::new(-4., 0., -4.),
                Vec3::new(0.0, 0.0, 10.0),
                Vec3::new(2., 0.0, -4.0),
            ]),
            super::QuadraticBezierCurve::new([
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0., 0.0, 0.0),
            ]),
        ];
        for curve in curves.iter() {
            let new_curve = curve.to_curve2(2., 1e6); // (equals to 5.7 degree)
            let len = new_curve.length();
            let real_len = curve.length();
            assert!(
                (len - real_len).abs() < 1e-1,
                "curve length: {:?}, real length: {:?}",
                len,
                real_len,
            );
            println!("segments: {:?}", new_curve.curves.len());
        }
    }

    #[test]
    fn test_divide_with_length() {
        // construct a extremely sharp curve
        let curves = [super::QuadraticBezierCurve::new([
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(1., 0.0, -10.0),
        ])];
        for curve in curves.iter() {
            let new_curve = curve.to_curve2(1e6, 3.0); // (equals to 5.7 degree)
            let len = new_curve.length();
            let real_len = curve.length();
            assert!(
                (len - real_len).abs() < 1e-1,
                "curve length: {:?}, real length: {:?}",
                len,
                real_len,
            );
            assert!(
                new_curve.curves.len() > 2,
                "curve length: {:?}, real length: {:?}",
                len,
                real_len,
            );
        }
    }
}
