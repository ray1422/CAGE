use bevy::{math::vec2, prelude::*};
mod plugins;
use cage::core::math::curve::{CageBezierCurve, CageCurve};
use plugins::{CageCameraPlugin /*RoadPlugin*/};

fn test_system(mut gizmos: Gizmos) {
    let curve = CageBezierCurve::new([
        // Vec3::new(-4.0, 0.0, -4.0),
        // Vec3::new(-2.0, 0.0, 4.0),
        // Vec3::new(4.0, 0.0, -4.0),
        // Vec3::new(4.0, 0.0, 4.0),
        Vec3::new(-8.0, 0.0, -8.0),
        Vec3::new(-4.8, 0.0, 6.0),
        Vec3::new(0.0, 0.0, -0.0),
        Vec3::new(8.0, 0.0, 8.0),
    ]);
    let new_curve = curve.shift(vec2(1.5, -1.0));
    let new_curve_r = curve.shift(vec2(-1.5, 1.0));
    curve
        .iter_positions(1024)
        .collect::<Vec<Vec3>>()
        .windows(2)
        .for_each(|p| gizmos.line(p[0], p[1], Color::WHITE));
    curve
        .control_points
        .windows(2)
        .for_each(|p| gizmos.line(p[0], p[1], Color::RED));

    new_curve
        .iter_positions(1024)
        .collect::<Vec<Vec3>>()
        .windows(2)
        .for_each(|p| gizmos.line(p[0], p[1], Color::BLACK));

    // new_curve
    //     .control_points
    //     .windows(2)
    //     .for_each(|p| gizmos.line(p[0], p[1], Color::RED));

    new_curve_r
        .iter_positions(1024)
        .collect::<Vec<Vec3>>()
        .windows(2)
        .for_each(|p| gizmos.line(p[0], p[1], Color::BLUE));

    // new_curve_r
    //     .control_points
    //     .windows(2)
    //     .for_each(|p| gizmos.line(p[0], p[1], Color::RED));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CageCameraPlugin)
        .add_systems(Update, test_system)
        // .add_plugins(RoadPlugin)
        .run();
}
