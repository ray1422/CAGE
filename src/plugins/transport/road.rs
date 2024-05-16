use std::{
    cmp::Ordering,
    collections::{self, BTreeMap},
};

use bevy::{
    prelude::*,
    utils::{petgraph::data::Build, HashMap, HashSet},
};
use cage::core::math::curve::{quadratic::QuadraticBezierCurve, Curve};

use crate::plugins::{camera::Ground, transport::path::Path};

use super::path::{self, PathLock, PathNext};

#[derive(Component, Clone)]
pub struct Road {
    pub center: Curve,
    pub width: f32,
    /// m/s
    pub speed_max: f32,
    pub travel_time_avg: f32,
}

impl Road {
    pub fn length(&self) -> f32 {
        self.center.length()
    }
    // m/s
    pub fn avg_speed(&self) -> f32 {
        self.length() / self.travel_time_avg
    }

    pub fn intersects(&self, rhs: &Road) -> Option<Vec3> {
        self.center
            .iter_positions(1024)
            .filter_map(|p| {
                let dist = rhs.center.distance_to(p);
                if dist < self.width || dist < rhs.width {
                    Some((dist, p))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Less))
            .map(|(_, p)| p)
    }
}

#[derive(Component, Clone)]
pub struct Junction {
    pub center: Vec3,
}

#[derive(Resource, Debug)]
/// road index is used to query near or collided roads.
pub struct RoadIndex {
    pub roads: HashSet<Entity>,
    pub junctions: HashSet<Entity>,
}
pub enum collisionTarget {
    Road(Entity),
    Junction(Entity),
}

impl RoadIndex {
    pub fn new() -> Self {
        Self {
            roads: HashSet::new(),
            junctions: HashSet::new(),
        }
    }
    pub fn add_road(&mut self, e: Entity) {
        self.roads.insert(e);
    }

    pub fn collisions(
        &self,
        _px: f32,
        _py: f32,
        _qx: f32,
        _qy: f32,
    ) -> impl Iterator<Item = collisionTarget> + '_ {
        self.roads
            .iter()
            .map(|r| collisionTarget::Road(*r))
            .chain(self.junctions.iter().map(|r| collisionTarget::Junction(*r)))
    }

    fn remove(&mut self, road: Entity) {
        self.roads.remove(&road);
        self.junctions.remove(&road);
    }
}

#[derive(Event, Clone, Debug)]
pub struct BuildRoad {
    pub center: Curve,
    pub width: f32,
    pub speed_max: f32,
}

#[derive(Clone, Debug)]
pub struct RoadBlueprint {
    pub event: BuildRoad,
    /// paths: form, owned, next
    pub paths: Vec<(Option<Entity>, Path, Option<Entity>)>,
}

impl RoadBlueprint {
    pub fn to_road(&self) -> Road {
        Road {
            center: self.event.center.clone(),
            width: self.event.width,
            speed_max: 2.,
            travel_time_avg: 1.,
        }
    }
}

pub struct JunctionBluePrint {
    center: Vec3,
    connections: Vec<(Option<Entity>, Path, Option<Entity>)>,
}
impl JunctionBluePrint {
    pub fn new(center: Vec3) -> Self {
        JunctionBluePrint {
            center,
            connections: Vec::new(),
        }
    }
}

/// split road into two road segment.
///
/// note that path hasn't spawn since blueprint isn't constructed yet.
fn split_road(
    bp: RoadBlueprint,
    at: Vec3,
) -> Result<(RoadBlueprint, JunctionBluePrint, RoadBlueprint), ()> {
    let curve = bp.event.center;
    let (curve_a, curve_b) = curve.split_at(at);

    let curve_a = curve_a.slice_by_length(0.0, curve_a.length() - bp.event.width / 2.)?;
    let curve_b = curve_b.slice_by_length(bp.event.width / 2., curve_b.length())?;
    let mut junction_bp = JunctionBluePrint::new(curve_a.end());
    let mut road_a_bp = RoadBlueprint {
        event: BuildRoad {
            center: curve_a,
            width: bp.event.width,
            speed_max: bp.event.speed_max,
        },
        paths: vec![],
    };
    let mut road_b_bp = RoadBlueprint {
        event: BuildRoad {
            center: curve_b,
            width: bp.event.width,
            speed_max: bp.event.speed_max,
        },
        paths: vec![],
    };

    for (from_e, path, next_e) in bp.paths {
        let (curve_a, curve_b) = path.curve.split_at(at);
        let curve_a = curve_a.slice_by_length(0.0, curve_a.length() - bp.event.width / 2.)?;
        let curve_b = curve_b.slice_by_length(bp.event.width / 2., curve_b.length())?;
        let junction_curve = Curve::form_two_velocity(
            curve_a.end(),
            curve_a.velocity(1.0),
            curve_b.start(),
            curve_b.velocity(0.0),
        );
        road_a_bp.paths.push((
            None,
            Path {
                curve: curve_a,
                left: None,
                right: None,
                ..default()
            },
            None,
        ));
        road_b_bp.paths.push((
            None,
            Path {
                curve: curve_b,
                left: None,
                right: None,
                ..default()
            },
            None,
        ));
        junction_bp.connections.push((
            None,
            Path {
                curve: junction_curve,
                left: None,
                right: None,
                ..default()
            },
            None,
        ))
    }
    Ok((road_a_bp, junction_bp, road_b_bp))
}

fn split_road_with_existing_junction(
    bp: RoadBlueprint,
    mut junction_bp: JunctionBluePrint,
    at: Vec3,
) -> Result<(RoadBlueprint, JunctionBluePrint, RoadBlueprint), ()> {
    let curve = bp.event.center;
    let (curve_a, curve_b) = curve.split_at(at);

    let curve_a = curve_a.slice_by_length(0.0, curve_a.length() - bp.event.width / 2.)?;
    let curve_b = curve_b.slice_by_length(bp.event.width / 2., curve_b.length())?;

    let mut road_a_bp = RoadBlueprint {
        event: BuildRoad {
            center: curve_a,
            width: bp.event.width,
            speed_max: bp.event.speed_max,
        },
        paths: vec![],
    };
    let mut road_b_bp = RoadBlueprint {
        event: BuildRoad {
            center: curve_b,
            width: bp.event.width,
            speed_max: bp.event.speed_max,
        },
        paths: vec![],
    };

    for (from_e, path, next_e) in bp.paths {
        let (curve_a, curve_b) = path.curve.split_at(at);
        let curve_a = curve_a.slice_by_length(0.0, curve_a.length() - bp.event.width / 2.)?;
        let curve_b = curve_b.slice_by_length(bp.event.width / 2., curve_b.length())?;
        let junction_curve = Curve::form_two_velocity(
            curve_a.end(),
            curve_a.velocity(1.0),
            curve_b.start(),
            curve_b.velocity(0.0),
        );
        road_a_bp.paths.push((
            None,
            Path {
                curve: curve_a,
                left: None,
                right: None,
                ..default()
            },
            None,
        ));
        road_b_bp.paths.push((
            None,
            Path {
                curve: curve_b,
                left: None,
                right: None,
                ..default()
            },
            None,
        ));
        junction_bp.connections.push((
            None,
            Path {
                curve: junction_curve,
                left: None,
                right: None,
                ..default()
            },
            None,
        ))
    }
    Ok((road_a_bp, junction_bp, road_b_bp))
}

fn split_collision_roads(
    mut commands: &mut Commands,
    mut roads: Vec<RoadBlueprint>,
    junctions: Vec<Junction>,
) -> Result<(Vec<RoadBlueprint>, Vec<Junction>), ()> {
    let mut flg = false;
    let mut new_roads: Vec<RoadBlueprint> = vec![];
    let mut new_junctions: Vec<RoadBlueprint> = vec![];
    for (i, road) in roads.into_iter().enumerate() {
        let mut flg_2 = false;
        for road_other in &new_roads {
            if let Some(pt) = road.to_road().intersects(&road_other.to_road()) {
                flg_2 = true;
                let (road_a, junction, road_b) = split_road(road.clone(), pt)?;
                let (road_c, junction, road_d) =
                    split_road_with_existing_junction(road_other.clone(), junction, pt)?;
            }
        }
    }
    todo!()
}

fn spawn_road(mut commands: &mut Commands, road_index: &mut RoadIndex, bp: RoadBlueprint) {
    let paths = bp.paths;

    let event = bp.event;
    let road_e = commands
        .spawn(Road {
            center: event.center.clone(),
            width: event.width,
            speed_max: event.speed_max,
            travel_time_avg: 1.,
        })
        .id();

    for (from_e, path, next_e) in paths {
        let path_e = commands.spawn(path).id();
        from_e.and_then(|from_e| path::add_next(&mut commands, from_e, path_e));
        next_e.and_then(|next_e| path::add_next(&mut commands, path_e, next_e));
    }

    road_index.add_road(road_e);
}

/// connections: ((P, V), (Q, U)) where P, Q is in and out points,
/// V and U is direction vector. Entity is path entity
fn spawn_junction(mut commands: &mut Commands, road_index: &mut RoadIndex, bp: JunctionBluePrint) {
    let (center, connections) = (bp.center, bp.connections);
    let junction_e = commands.spawn(Junction { center }).id();

    for (from_path_e, path, next_path_e) in connections {
        let path_e = commands.spawn(path).id();
        next_path_e.and_then(|next_path_e| path::add_next(&mut commands, path_e, next_path_e));
        from_path_e.and_then(|from_path_e| path::add_next(&mut commands, from_path_e, path_e));
        commands.entity(junction_e).add_child(path_e);
    }
}

fn build_road_system(
    mut commands: Commands,
    mut road_index: ResMut<RoadIndex>,
    mut road_query: Query<&mut Road>,
    mut path_query: Query<&mut Path>,
    mut events: EventReader<BuildRoad>,
) {
    for event in events.read() {
        // check if collision
        // if collision, split existing road into two road and junction
        // and spawn two road, connect them to junction
        // else just spawn road with no connection
        spawn_road(&mut commands, &mut road_index, RoadBlueprint{ event: todo!(), paths: todo!() });
    }
}

pub fn show_debug_road(roads: Query<&Road>, mut gizmos: Gizmos) {
    for road in roads.iter() {
        road.center
            .iter_positions(64)
            .collect::<Vec<Vec3>>()
            .windows(2)
            .for_each(|p| gizmos.line(p[0], p[1], Color::BLACK));
    }
}

#[derive(Resource)]
struct RoadBuildingState {
    pts: Vec<Vec3>,
}

impl RoadBuildingState {
    fn new() -> Self {
        Self { pts: Vec::new() }
    }
}

pub struct RoadBuildingPlugin;

fn build_road_building_system(
    mut state: ResMut<RoadBuildingState>,
    mut events: EventWriter<BuildRoad>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_event: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };
    let (camera, camera_transform) = camera_query.single();
    let ground = ground_query.single();

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };
    let Some(distance) = ray.intersect_plane(ground.translation(), Plane3d::new(ground.up()))
    else {
        return;
    };
    let point = ray.get_point(distance);
    if (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight))
        && mouse_event.just_pressed(MouseButton::Left)
    {
        state.pts.push(point);
    }
    if keys.pressed(KeyCode::Escape) {
        state.pts.clear();
    }
    if state.pts.len() == 3 {
        // send event to build road
        let curve = QuadraticBezierCurve::new([state.pts[0], state.pts[1], state.pts[2]]);
        let width = 2.0;
        let speed_max = 10.;
        events.send(BuildRoad {
            center: curve.to_curve(),
            width,
            speed_max,
        });
        state.pts.clear();
    }
    state.pts.windows(2).for_each(|pts| {
        gizmos.line(pts[0], pts[1], Color::RED);
    });
}

impl Plugin for RoadBuildingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RoadBuildingState::new());
        app.add_systems(Update, build_road_building_system);
    }
}

pub struct RoadPlugin;

impl Plugin for RoadPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RoadIndex::new());
        app.add_event::<BuildRoad>();
        app.add_systems(PostUpdate, build_road_system);
    }
}
