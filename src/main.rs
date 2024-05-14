use bevy::{
    math::{vec2, vec3},
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages},
    transform::{self, commands},
};
mod plugins;
use bevy::render::render_resource::PrimitiveTopology::TriangleList;

use cage::core::math::curve::{quadratic::QuadraticBezierCurve, Curve};
use plugins::{CageCameraPlugin /*RoadPlugin*/};
// fn test_mesh(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let curve = CageBezierCurve::new([
//         Vec3::new(-8.0, 0.0, -8.0),
//         Vec3::new(-4.8, 0.0, 6.0),
//         Vec3::new(0.0, 0.0, -0.0),
//         Vec3::new(8.0, 0.0, 8.0),
//     ]);
//     let new_curve = curve.offset(vec2(1.5, 0.));
//     let new_curve_r = curve.offset(vec2(-1.5, 0.));
//     // generate vertices
//     // let vertices = curve.iter_positions(1024).collect::<Vec<Vec3>>();

//     let mut vertices: Vec<Vec3> = Vec::new();
//     for p in new_curve.iter_positions(1024) {
//         vertices.push(p);
//     }
//     for p in new_curve_r.iter_positions(1024) {
//         vertices.push(p);
//     }

//     // generate indices
//     let mut indices = Vec::new();

//     for i in 0..1023 {
//         indices.push(i as u32);
//         indices.push((i + 1) as u32);
//         indices.push((i + 1024) as u32);
//         indices.push((i + 1024) as u32);
//         indices.push((i + 1) as u32);
//         indices.push((i + 1025) as u32);
//     }

//     let mut mesh = Mesh::new(TriangleList, RenderAssetUsages::default());
//     mesh.insert_attribute(
//         Mesh::ATTRIBUTE_NORMAL,
//         vec![Vec3::new(0., 1., 0.)].repeat((&vertices).len()),
//     );
//     mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
//     mesh.insert_indices(Indices::U32(indices));

//     let asset = Color::DARK_GRAY;

//     commands.spawn(PbrBundle {
//         mesh: meshes.add(mesh),
//         material: materials.add(asset),
//         transform: Transform::from_translation(vec3(0., 0.1, 0.)),
//         ..Default::default()
//     });
// }

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
    gizmos.line(Vec3::new(-9., 0.0, -9.0), Vec3::new(-5.0, 0.0, 8.0), Color::WHITE);
    gizmos.line(Vec3::new(-5.0, 0.0, 8.0), Vec3::new(5.0, 0.0, -9.0), Color::WHITE);
    gizmos.line(Vec3::new(5.0, 0.0, -9.0), Vec3::new(9., 0.0, -7.0), Color::WHITE);
    

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
