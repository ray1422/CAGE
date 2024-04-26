use bevy::prelude::*;

/// this file is the example of creating a cube in the scene

pub struct CubePlugin;
impl Plugin for CubePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

#[derive(Component)]
struct Cube;

#[derive(Bundle)]
struct CubeBundle {
    cube: Cube,
    transform: Transform,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::from_size(Vec3::ONE)),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
        ..Default::default()
    });
}
