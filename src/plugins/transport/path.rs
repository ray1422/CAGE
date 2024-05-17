use bevy::prelude::*;
use cage::core::math::curve::{quadratic::QuadraticBezierCurve, Curve};

#[derive(Component, Debug, Clone)]
pub struct Path {
    pub curve: Curve,
    // left Path entity
    pub left: Option<Entity>,
    // right Path entity
    pub right: Option<Entity>,
    // locks and owners
    pub locks: Vec<PathLock>,
}

#[derive(Component, Debug, Clone)]
pub struct PathNext {
    pub next: Entity,
}

#[derive(Component)]
pub struct PathPrev {
    pub prev: Entity,
}

impl Default for Path {
    fn default() -> Self {
        Self {
            // no default
            curve: QuadraticBezierCurve::new([Vec3::ZERO, Vec3::ZERO, Vec3::ZERO]).to_curve(),
            left: None,
            right: None,
            locks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PathLockStatus {
    Pending,
    Locked,
}

#[derive(Component, Debug, Clone)]
pub struct PathLock {
    pub priority: i16,
    pub status: PathLockStatus,
    pub owner: Entity,
    // timestamp of ms when the path is locked
    // pub est_available_in: u64,
    pub start: f32,
    pub end: f32,
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
    dst_path_e: Entity,
) -> Option<(Entity, Entity)> {
    commands
        .get_entity(path_e)
        .and_then(|mut path_ec| {
            let next = path_ec
                .commands()
                .spawn(PathNext { next: dst_path_e })
                .set_parent(path_e)
                .id();
            path_ec.add_child(next);

            Some(next)
        })
        .and_then(|next_ret| commands.get_entity(dst_path_e).map(|e| (next_ret, e)))
        .and_then(|(next_ret, mut dst_path_ec)| {
            let prev = dst_path_ec.commands().spawn(PathPrev { prev: path_e }).id();
            Some((next_ret, prev))
        })
}
