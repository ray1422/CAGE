use bevy::prelude::*;
mod plugins;

use cage::core::math::curve::Curve;
use plugins::{CageCameraPlugin /*RoadPlugin*/};

fn test_system(mut gizmos: Gizmos) {
    // let curve = QuadraticBezierCurve::new([
    //     Vec3::new(-9., 0.0, -2.0),
    //     Vec3::new(8.0, 0.0, 8.0),
    //     Vec3::new(2.0, 0.0, 2.0),
    // ]);
    let curve = Curve::from_4_points(
        Vec3::new(-9., 0.0, -9.0),
        Vec3::new(-5.0, 0.0, 8.0),
        Vec3::new(5.0, 0.0, -9.0),
        Vec3::new(9., 0.0, -7.0),
    );
    // plot line segments of 4 points

    gizmos.line(
        Vec3::new(-9., 0.0, -9.0),
        Vec3::new(-5.0, 0.0, 8.0),
        Color::WHITE,
    );

    let curve_2 = curve.offset(2., 0.);

    curve
        .iter_positions(1024)
        .collect::<Vec<Vec3>>()
        .windows(2)
        .for_each(|p| gizmos.line(p[0], p[1], Color::BLUE));

    curve_2
        .iter_positions(1024)
        .collect::<Vec<Vec3>>()
        .windows(2)
        .for_each(|p| {
            gizmos.line(p[0], p[1], Color::RED);
        });

    let p = curve.position(0.3);
    let v = curve.velocity(0.3);
    let q = curve_2.position(0.7);
    let u = curve_2.velocity(0.7);
    // point p to q
    gizmos.line(p, q, Color::YELLOW);
    let new_curve = Curve::form_two_velocity(p, v, q, u);
    // plot new curve
    new_curve
        .iter_positions(1024)
        .collect::<Vec<Vec3>>()
        .windows(2)
        .for_each(|p| gizmos.line(p[0], p[1], Color::GREEN));

    // new_curve
    //     .control_points
    //     .windows(2)
    //     .for_each(|p| gizmos.line(p[0], p[1], Color::RED));

    // new_curve_r
    //     .iter_positions(1024)
    //     .collect::<Vec<Vec3>>()
    //     .windows(2)
    //     .for_each(|p| gizmos.line(p[0], p[1], Color::BLUE));

    // new_curve_r
    //     .control_points
    //     .windows(2)
    //     .for_each(|p| gizmos.line(p[0], p[1], Color::RED));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_systems(Startup, test_mesh)
        .add_systems(Update, test_system)
        .add_plugins(CageCameraPlugin)
        // .add_plugins(RoadPlugin)
        .run();
}
