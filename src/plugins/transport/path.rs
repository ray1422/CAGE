use bevy::prelude::*;
use cage::core::math::curve::{quadratic::QuadraticBezierCurve, Curve};

use super::path_op::{schedule_intents, PathLockIndex};

#[derive(Component, Debug, Clone)]
pub struct Path {
    pub curve: Curve,
    // left Path entity
    pub left: Option<Entity>,
    // right Path entity
    pub right: Option<Entity>,
}
impl Path {
    pub fn new(curve: Curve) -> Self {
        Self {
            curve,
            left: None,
            right: None,
        }
    }
    pub fn length(&self) -> f32 {
        self.curve.length()
    }
}

#[derive(Component, Debug, Clone)]
pub struct PathNext {
    /// keep driving until |this_until| on this path
    this_until: f32,
    pub next: Entity,
    /// after |this_to|, start driving from |next_from| on next path
    next_from: f32,
}

#[derive(Component)]
pub struct PathPrev {
    /// the point that starts driving on this path
    this_from: f32,
    pub prev: Entity,
    /// the point that ends driving on prev path
    prev_until: f32,
}

impl Default for Path {
    fn default() -> Self {
        Self {
            // no default
            curve: QuadraticBezierCurve::new([Vec3::ZERO, Vec3::ZERO, Vec3::ZERO]).to_curve(),
            left: None,
            right: None,
        }
    }
}

pub fn show_debug_path(paths: Query<&Path>, mut gizmos: Gizmos) {
    for path in paths.iter() {
        path.curve
            .iter_positions(64)
            .collect::<Vec<Vec3>>()
            .windows(2)
            .for_each(|p| gizmos.line(p[0], p[1], Color::WHITE));
        let src = path.curve.position(0.0);
        let dst = path.curve.position(1.0);
        gizmos.circle(src, Direction3d::Y, 0.05, Color::GREEN);
        gizmos.circle(dst + Vec3::Y * 0.05, Direction3d::Y, 0.05, Color::RED);
    }
}

/// add next entity to |path_e|. return next_path_e and reversed prev entity
pub fn link_next(
    commands: &mut Commands,
    path_e: Entity,
    this_until: f32,
    dst_path_e: Entity,
    next_from: f32,
) -> Option<(Entity, Entity)> {
    commands
        .get_entity(path_e)
        .and_then(|mut path_ec| {
            let next = path_ec
                .commands()
                .spawn(PathNext {
                    next: dst_path_e,
                    this_until,
                    next_from,
                })
                .set_parent(path_e)
                .id();
            path_ec.add_child(next);

            Some(next)
        })
        .and_then(|next_ret| commands.get_entity(dst_path_e).map(|e| (next_ret, e)))
        .and_then(|(next_ret, mut dst_path_ec)| {
            let prev = dst_path_ec
                .commands()
                .spawn(PathPrev {
                    prev: path_e,
                    this_from: next_from,
                    prev_until: this_until,
                })
                .id();
            Some((next_ret, prev))
        })
}

pub struct PathPlugin;

impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PathLockIndex::new())
            .add_systems(Update, schedule_intents);
        // app.add_startup_system(test_setup_path.system())
        //     .add_system(show_debug_path.system());
    }
}
