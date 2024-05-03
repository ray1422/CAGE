use bevy::{
    math::vec2,
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages},
    transform::commands,
};
mod plugins;
use bevy::render::render_resource::PrimitiveTopology::TriangleList;
use cage::core::math::curve::{CageBezierCurve, CageCurve};
use peroxide::fuga::DType::U32;
use plugins::{CageCameraPlugin /*RoadPlugin*/};
fn test_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
    let new_curve = curve.shift(vec2(1.5, 1.0));
    let new_curve_r = curve.shift(vec2(-1.5, 1.0));
    // generate vertices
    // let vertices = curve.iter_positions(1024).collect::<Vec<Vec3>>();

    let mut vertices: Vec<Vec3> = Vec::new();
    for p in new_curve.iter_positions(1024) {
        vertices.push(p);
    }
    for p in new_curve_r.iter_positions(1024) {
        vertices.push(p);
    }

    // generate indices
    let mut indices = Vec::new();

    for i in 0..1023 {
        if vertices[i] == Vec3::ZERO
            || vertices[i + 1] == Vec3::ZERO
            || vertices[i + 1024] == Vec3::ZERO
            || vertices[i + 1025] == Vec3::ZERO
        {
            continue;
        }
        indices.push(i as u32);
        indices.push((i + 1) as u32);
        indices.push((i + 1024) as u32);
        indices.push((i + 1024) as u32);
        indices.push((i + 1) as u32);
        indices.push((i + 1025) as u32);
    }
    let mut mesh = Mesh::new(TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));

    let mesh = meshes.add(mesh);
    commands.spawn(PbrBundle {
        mesh,
        material: materials.add(Color::WHITE),
        ..Default::default()
    });
}

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
        .add_systems(Startup, test_mesh)
        .add_systems(Update, test_system)
        // .add_plugins(RoadPlugin)
        .run();
}
