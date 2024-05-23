#![feature(btree_cursors)]

use bevy::prelude::*;
mod plugins;

use cage::core::math::curve::Curve;
use plugins::{
    transport::{
        car::{car_intents_lock, car_intent_update, car_move, test_setup_car_and_path},
        path::{show_debug_path, PathPlugin},
        road::RoadBuildingPlugin,
    },
    CageCameraPlugin, RoadPlugin, /*RoadPlugin*/
};

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

    // let (curve_a, curve_b) = curve.split_at(Vec3::new(5., 0., 3.));
    let (curve_a, curve_b) = curve.split_at(Vec3::new(-9., 0.0, -9.0));

    curve_a
        .iter_positions(1024)
        .collect::<Vec<Vec3>>()
        .windows(2)
        .for_each(|p| gizmos.line(p[0], p[1], Color::CYAN));
    curve_b
        .iter_positions(1024)
        .collect::<Vec<Vec3>>()
        .windows(2)
        .for_each(|p| gizmos.line(p[0], p[1], Color::PINK));
    return;
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
    let new_curve = Curve::form_two_velocity(p, v, q, u).unwrap();
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
        .add_plugins(PathPlugin)
        // .add_systems(Startup, test_mesh)
        .add_systems(Startup, test_setup_car_and_path)
        // .add_systems(Update, test_system)
        .add_systems(Update, show_debug_path)
        .add_systems(Update, (car_intents_lock, car_move, car_intent_update))
        .add_plugins(CageCameraPlugin)
        .add_plugins(RoadPlugin)
        .add_plugins(RoadBuildingPlugin)
        // .add_plugins(RoadPlugin)
        .run();
}
