use bevy::{math::vec3, prelude::*};

pub struct RoadPlugin;

impl Plugin for RoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

#[derive(Component)]
struct Road;

#[derive(Bundle)]
struct RoadBundle {
    road: Road,
    pbr_bundle: PbrBundle,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let control_points = [[
        Vec3::new(-10.0, 0.0, -10.0),
        Vec3::new(5.0, 0.0, 1.0),
        Vec3::new(2.0, 0.0, 2.0),
        Vec3::new(10.0, 0.0, 10.),
    ]];
    // mesh: curve with radius 1.0

    let curve: bevy::math::cubic_splines::CubicCurve<Vec3> =
        CubicBezier::new(control_points).to_curve();

    let points = curve.iter_positions(64).collect::<Vec<_>>();
    // define the mesh by the points
    // get polygon over the points
    let cuboid = Cuboid::default();
    let mesh = meshes.add(cuboid.mesh());
    for i in 0..points.len() - 1 {
        let p0 = points[i];
        let p1 = points[i + 1];

        // make the direction of the cuboid to be the direction of the curve
        let transform = Transform::from_translation(p0)
            * Transform::from_scale(vec3((p1 - p0).length() + 0.1, 1.0, 0.1))
            * {
                let direction = p1 - p0;
                let rotation = Quat::from_rotation_arc(Vec3::X, direction.normalize());
                Transform::from_rotation(rotation)
            };

        let material = materials.add(Color::rgb(0.8, 0.7, 0.6));
        commands.spawn(RoadBundle {
            road: Road,

            pbr_bundle: PbrBundle {
                mesh: mesh.clone(),
                material,
                transform,
                ..Default::default()
            },
        });
    }
}
