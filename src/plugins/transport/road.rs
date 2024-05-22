use anyhow::Result;
use anyhow::anyhow;
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use cage::core::math::curve::{quadratic::QuadraticBezierCurve, Curve};
use std::{cmp::Ordering, vec};

use crate::plugins::{camera::Ground, transport::path::Path};

use super::path::{self, PathNext, PathPrev};

#[derive(Component, Clone, Debug)]
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
pub enum CollisionTarget {
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

    pub fn add_junction(&mut self, e: Entity) {
        self.junctions.insert(e);
    }

    pub fn collisions(
        &self,
        _px: f32,
        _py: f32,
        _qx: f32,
        _qy: f32,
    ) -> impl Iterator<Item = CollisionTarget> + '_ {
        self.roads
            .iter()
            .map(|r| CollisionTarget::Road(*r))
            .chain(self.junctions.iter().map(|r| CollisionTarget::Junction(*r)))
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
) -> Result<(RoadBlueprint, JunctionBluePrint, RoadBlueprint)> {
    let curve = bp.event.center;
    let (curve_a, curve_b) = curve.split_at(at);

    let curve_a = curve_a.slice_by_length(0.0, curve_a.length() - bp.event.width / 2.)?;
    let curve_b = curve_b.slice_by_length(bp.event.width / 2., curve_b.length())?;
    let junction_bp = JunctionBluePrint::new(curve_a.end());
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

        road_a_bp.paths.push((
            from_e,
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
            next_e,
        ));
    }
    Ok((road_a_bp, junction_bp, road_b_bp))
}

fn split_road_with_existing_junction(
    bp: RoadBlueprint,
    mut junction_bp: JunctionBluePrint,
    at: Vec3,
) -> Result<(RoadBlueprint, JunctionBluePrint, RoadBlueprint)> {
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
        road_a_bp.paths.push((
            from_e,
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
            next_e,
        ));
    }
    Ok((road_a_bp, junction_bp, road_b_bp))
}

/// p, v is path_a's entering position and direction
/// q, u is path_b's outgoing position and direction
fn connect_road_in_junction(
    mut commands: &mut Commands,
    path_a_e: Entity,
    path_b_e: Entity,
    junction_e: Entity,
    p: Vec3,
    v: Vec3,
    q: Vec3,
    u: Vec3,
) -> Result<()> {
    let curve = Curve::form_two_velocity(p, v, q, u)?;
    commands.get_entity(junction_e).map_or(Err(anyhow!("entity not found")), |mut j_ec| {
        j_ec.with_children(|parent| {
            parent.spawn(Path {
                curve,
                left: None,
                right: None,
                ..default()
            });
        });
        Ok(())
    })
}

fn spawn_junction_full_connections(
    incoming_groups: Vec<Vec<Path>>,
    outgoing_groups: Vec<Vec<Path>>,
) -> Result<HashMap<(usize, usize, usize, usize), Path>> {
    let mut ret = HashMap::<(usize, usize, usize, usize), Path>::new();
    for (i, ips) in incoming_groups.iter().enumerate() {
        for (o, ops) in outgoing_groups.iter().enumerate() {
            for (i2, ip) in ips.iter().enumerate() {
                for (o2, op) in ops.iter().enumerate() {
                    ret.insert(
                        (i, i2, o, o2),
                        Path {
                            curve: Curve::form_two_velocity(
                                ip.curve.end(),
                                ip.curve.velocity(1.0),
                                op.curve.start(),
                                op.curve.velocity(0.0),
                            )?,
                            ..default()
                        },
                    );
                }
            }
        }
    }
    Ok(ret)
}

/// this function despawn old entities and spawn new entities replacing collision roads with
/// new roads with junctions.
fn spawn_split_collision_roads(
    mut commands: &mut Commands,
    roads: Vec<RoadBlueprint>,
    mut road_index: &mut ResMut<RoadIndex>,
) -> Result<()> {
    let mut new_roads: Vec<RoadBlueprint> = vec![];
    for road in roads.into_iter() {
        let mut rm_idx = HashSet::<usize>::new();
        for (j, road_other) in new_roads.iter().enumerate() {
            if (road.event.center.start() - road_other.event.center.start()).length() < 1e-6
                && (road.event.center.end() - road_other.event.center.end()).length() < 1e-6
            {
                continue;
            }
            if let Some(pt) = road.to_road().intersects(&road_other.to_road()) {
                rm_idx.insert(j);

                let (road_a, junction, road_b) = split_road(road.clone(), pt)?;
                let (road_c, junction, road_d) =
                    split_road_with_existing_junction(road_other.clone(), junction, pt)?;
                println!(
                    "new roads\n \t{:?}\n\t{:?}\n\t{:?}\n\t{:?}\n",
                    road_a, road_b, road_c, road_d
                );
                let (road_a_e, paths_a_e) = spawn_road(&mut commands, &mut road_index, &road_a);
                let (road_b_e, paths_b_e) = spawn_road(&mut commands, &mut road_index, &road_b);
                let (road_c_e, paths_c_e) = spawn_road(&mut commands, &mut road_index, &road_c);
                let (road_d_e, paths_d_e) = spawn_road(&mut commands, &mut road_index, &road_d);
                let get_second = |e: &(Option<Entity>, Path, Option<Entity>)| e.1.clone();
                let rg1 = [
                    road_a.paths.iter().map(get_second).collect::<Vec<Path>>(),
                    road_c.paths.iter().map(get_second).collect::<Vec<Path>>(),
                ]
                .to_vec();
                let rg2 = [
                    road_b.paths.iter().map(get_second).collect::<Vec<Path>>(),
                    road_d.paths.iter().map(get_second).collect::<Vec<Path>>(),
                ]
                .to_vec();

                // generate paths to connect paths_a to b, d
                // generate paths to connect paths_c to b, d
                let ret = spawn_junction_full_connections(rg1, rg2)?;
                let i_roads = [road_a_e, road_c_e];
                let o_roads = [road_b_e, road_d_e];
                let i_paths = [paths_a_e, paths_c_e];
                let o_paths = [paths_b_e, paths_d_e];
                let junction_e = spawn_junction(
                    commands,
                    road_index,
                    JunctionBluePrint {
                        center: pt,
                        connections: ret
                            .into_iter()
                            .map(|((ir, ip, or, op), path)| {
                                (Some(i_paths[ir][ip]), path, Some(o_paths[or][op]))
                            })
                            .collect::<Vec<(Option<Entity>, Path, Option<Entity>)>>(),
                    },
                );
            }
        }
        if rm_idx.len() == 0 {
            new_roads.push(road);
        }
    }
    Ok(())
}

/// return road entity and paths entity
fn spawn_road(
    mut commands: &mut Commands,
    road_index: &mut RoadIndex,
    bp: &RoadBlueprint,
) -> (Entity, Vec<Entity>) {
    let paths = bp.paths.clone();
    let mut paths_entities = vec![];
    let event = bp.event.clone();
    let road_e = commands
        .spawn(Road {
            center: event.center.clone(),
            width: event.width,
            speed_max: event.speed_max,
            travel_time_avg: 1.,
        })
        .id();
    println!("road {:?} has been spawned", road_e);

    for (from_e, path, next_e) in paths {
        let path_e = commands.spawn(path).set_parent(road_e).id();
        commands.entity(road_e).add_child(path_e);
        paths_entities.push(path_e);
        from_e.and_then(|from_e| path::link_next(&mut commands, from_e, 1.0, path_e, 0.0));
        next_e.and_then(|next_e| path::link_next(&mut commands, path_e, 1.0, next_e, 0.0));
    }

    road_index.add_road(road_e);
    println!("road_index: {:?}", road_index);
    println!("spawn_road!");
    return (road_e, paths_entities);
}

/// connections: ((P, V), (Q, U)) where P, Q is in and out points,
/// V and U is direction vector. Entity is path entity
fn spawn_junction(
    mut commands: &mut Commands,
    road_index: &mut RoadIndex,
    bp: JunctionBluePrint,
) -> Entity {
    let (center, connections) = (bp.center, bp.connections);
    let junction_e = commands.spawn(Junction { center }).id();

    for (from_path_e, path, next_path_e) in connections {
        let path_e = commands.spawn(path).set_parent(junction_e).id();
        next_path_e
            .and_then(|next_path_e| path::link_next(&mut commands, path_e, 1.0, next_path_e, 0.0));
        from_path_e
            .and_then(|from_path_e| path::link_next(&mut commands, from_path_e, 1.0, path_e, 0.0));
        commands.entity(junction_e).add_child(path_e);
        commands.entity(path_e).set_parent(junction_e);
    }
    road_index.add_junction(junction_e);
    junction_e
}

fn build_road_system(
    mut commands: Commands,
    mut road_index: ResMut<RoadIndex>,
    road_query: Query<(&mut Road, Option<&Children>)>,
    // path_query: Query<(&mut Path, &Children)>,
    path_query: Query<(&mut Path, Option<&Children>)>,
    // path_query_2: Query<&mut Path>,
    next_query: Query<&PathNext>,
    prev_query: Query<&PathPrev>,
    mut events: EventReader<BuildRoad>,
) {
    for event in events.read() {
        let mut flg = false;
        // check if collision
        // if collision, split existing road into two road and junction
        // TODO: replace with real position
        let road_bp = &RoadBlueprint {
            event: event.clone(),
            paths: vec![(
                None,
                Path {
                    curve: event.center.clone(),
                    ..default()
                },
                None,
            )],
        };
        for other_road_e in road_index
            .collisions(0., 0., 0., 0.)
            .collect::<Vec<CollisionTarget>>()
        {
            match other_road_e {
                CollisionTarget::Road(old_road_e) => {
                    // check if it's collision
                    println!(
                        "{:?} should be road. actual: {:?}",
                        old_road_e,
                        road_query.get(old_road_e)
                    );

                    road_query
                        .get(old_road_e)
                        .ok()
                        .and_then(|(road_other, children)| {
                            let mut path_prev_next =
                                HashMap::<Entity, (Option<Entity>, Path, Option<Entity>)>::new();
                            if let Some(children) = children {
                                children.iter()
                            } else {
                                [].iter()
                            }
                            .filter_map(|path_e| {
                                let e = path_query.get(*path_e);
                                if let Ok(i) = e {
                                    println!("Some: {:?}", e);
                                    Some((path_e, i.0, i.1))
                                } else {
                                    println!("err: {:?}", e);
                                    None
                                }
                            })
                            .for_each(|(path_e, path, children)| {
                                let path_e = *path_e;
                                path_prev_next.insert(old_road_e, (None, path.clone(), None));
                                children.map_or_else(|| [].iter(), |e| e.iter()).for_each(
                                    |next_or_prev_e| {
                                        if next_query.get(*next_or_prev_e).is_ok()
                                            && path_prev_next.contains_key(&path_e)
                                        {
                                            let tmp = &path_prev_next[&path_e].clone();
                                            path_prev_next.remove(&path_e);
                                            path_prev_next.insert(
                                                path_e,
                                                (tmp.0, tmp.1.clone(), Some(*next_or_prev_e)),
                                            );
                                        } else if prev_query.get(*next_or_prev_e).is_ok() {
                                            let tmp = &path_prev_next[&path_e].clone();
                                            path_prev_next.remove(&path_e);
                                            path_prev_next.insert(
                                                path_e,
                                                (Some(*next_or_prev_e), tmp.1.clone(), tmp.2),
                                            );
                                        }
                                    },
                                )
                            });
                            road_bp.to_road().intersects(road_other).and_then(|_| {
                                flg = true;
                                Some((road_other, path_prev_next.into_iter().map(|(_, u)| u)))
                            })
                        })
                        .and_then(|(road_other, paths)| {
                            spawn_split_collision_roads(
                                &mut commands,
                                vec![
                                    RoadBlueprint {
                                        event: BuildRoad {
                                            center: road_other.center.clone(),
                                            width: road_other.width,
                                            speed_max: road_other.speed_max,
                                        },
                                        paths: paths.map(|p| p.clone()).collect(),
                                    },
                                    road_bp.clone(),
                                ],
                                &mut road_index,
                            )
                            .ok()
                        })
                        .and_then(|_| {
                            commands.entity(old_road_e).despawn_recursive();
                            road_index.remove(old_road_e);
                            Some(())
                        });
                }
                CollisionTarget::Junction(e) => {
                    // TODO
                }
            }
        }
        // and spawn two road, connect them to junction
        // else just spawn road with no connection
        if flg {
            continue;
        }
        spawn_road(
            &mut commands,
            &mut road_index,
            &RoadBlueprint {
                event: event.clone(),
                paths: vec![(
                    None,
                    Path {
                        curve: event.center.clone(),
                        ..default()
                    },
                    None,
                )],
            },
        );
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
