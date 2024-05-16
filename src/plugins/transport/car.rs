use bevy::{math::Vec3, pbr::PbrBundle, prelude::*, render::mesh::Mesh, time::Time};
use cage::core::math::curve::{quadratic::QuadraticBezierCurve, Curve};

use super::path::Path;

#[derive(Component, Debug)]
pub struct Car {
    speed: f32,
    path_idx: usize,
    // progress of the car on the current path
    progress: f32,
    pub paths: Vec<Entity>,
}

#[derive(Bundle)]
pub struct CarBundle {
    car: Car,
    pbr: PbrBundle,
}

pub fn drive_car(
    mut car_query: Query<(&mut Car, &mut Transform)>,
    path_query: Query<&Path>,
    time: Res<Time>,
) {
    for (mut car, mut transform) in car_query.iter_mut() {
        // move the car along the path
        let path = car.paths[car.path_idx];
        let mut path = path_query.get(path).unwrap();
        // query component path by entity
        let distance = car.speed * time.delta_seconds();
        car.progress += distance / path.curve.length();
        while car.progress >= 1.0 {
            let remaining_distance = (car.progress - 1.0) * path.curve.length();
            car.path_idx += 1;
            if car.path_idx >= car.paths.len() {
                car.path_idx = 0;
            }
            path = path_query.get(car.paths[car.path_idx]).unwrap();
            car.progress = remaining_distance / path.curve.length();
        }
        let position = path.curve.position(car.progress);
        let position_next = path.curve.position(car.progress + 0.01);
        let translate = Transform::from_translation(position);
        let rotation = translate.looking_at(position_next, Vec3::Y);
        // align the car's front/center/bottom with the path
        let shift = Transform::from_translation(Vec3::new(0.0, 0.5, 0.0));
        *transform = rotation * shift;
    }
}

pub fn test_setup_car_and_path(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let path = commands
        .spawn(Path {
            curve: Curve::from_4_points(
                Vec3::new(-9., 0.0, -9.0),
                Vec3::new(-5.0, 0.0, 8.0),
                Vec3::new(5.0, 0.0, -9.0),
                Vec3::new(6., 0.0, -7.0),
            ),
            left: None,
            right: None,
            locks: vec![],
        })
        .id();

    let path_conn = commands
        .spawn(Path {
            curve: Curve::form_two_velocity(
                Vec3::new(6., 0.0, -7.0),
                Vec3::new(1.0, 0.0, 2.0),
                Vec3::new(6.5, 0.0, -6.5),
                Vec3::new(1.5, 0.0, 8.0 + 6.5),
            ),
            left: None,
            right: None,
            locks: vec![],
        })
        .id();

    let path_2 = commands
        .spawn(Path {
            // smoothy connected from the last path
            curve: QuadraticBezierCurve::new([
                Vec3::new(6.5, 0.0, -6.5),
                Vec3::new(8.0, 0.0, 8.0),
                Vec3::new(9.0, 0.0, 9.0),
            ])
            .to_curve(),
            left: None,
            right: None,
            locks: vec![],
        })
        .id();

    commands.spawn(CarBundle {
        // a cube
        pbr: PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 2.0)),
            material: materials.add(Color::WHITE),

            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..Default::default()
        },
        car: Car {
            speed: 2.,
            path_idx: 0,
            progress: 0.0,
            paths: vec![path, path_conn, path_2],
        },
    });

    commands.spawn(CarBundle {
        // a cube
        pbr: PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 2.0)),
            material: materials.add(Color::WHITE),

            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..Default::default()
        },
        car: Car {
            speed: 2.,
            path_idx: 0,
            progress: 0.0,
            paths: vec![path, path_conn, path_2],
        },
    });
}
