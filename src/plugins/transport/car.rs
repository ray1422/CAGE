use std::collections::VecDeque;

use bevy::{
    math::Vec3, pbr::PbrBundle, prelude::*, render::mesh::Mesh, time::Time, utils::HashSet,
};
use cage::core::math::curve::quadratic::QuadraticBezierCurve;

use super::{
    path::{link_next, Path},
    path_op::{
        PathIntent, PathIntentApproved, PathLockIndex, PathLockTogether, PathSlice, PathSliceLock,
        PathSlicesLocked,
    },
};

#[derive(Component, Debug)]
pub struct Car {
    length: f32,
    speed: f32,
    last_position: Vec3,
    acceleration: f32,
    acc_max: f32,
    // entity of (pathSlice, Option<lock group>)
    pub path_slices: VecDeque<Entity>,
}

#[derive(Bundle)]
pub struct CarBundle {
    car: Car,
    intent: PathIntent,
    locks: PathSlicesLocked,
    pbr: PbrBundle,
}

fn update_one_car_intent(
    car: &Car,
    intent: &mut Mut<PathIntent>,
    path_slices_query: &mut Query<(&mut PathSlice, Option<&PathLockTogether>)>,
) {
    intent.path_locks.clear();
    // s = v0t + 0.5at^2
    // next 2s
    // TODO: should be 2.0_f32
    let mut dist = 2.0_f32
        .max(car.speed * 2.0)
        .max(0.5 * car.acceleration * 4.0 + car.speed * 2.0);
    for car_path_slice_e in car.path_slices.iter() {
        let ret = path_slices_query.get(*car_path_slice_e);
        if ret.is_err() {
            continue;
        }
        let (ps, other_locks) = ret.unwrap();
        let lock_together = other_locks.is_some();
        if dist <= 0.0 {
            break;
        }

        let ps_len = ps.length();
        if ps_len <= dist {
            dist -= ps_len;
            intent.path_locks.push_back(PathSliceLock {
                path_slice: ps.clone(),
                lock_together,
                is_main_path: true,
            });
        } else {
            intent.path_locks.push_back(PathSliceLock {
                path_slice: PathSlice::new(
                    ps.path_e,
                    ps.start,
                    ps.parent_t_of_length(dist),
                    ps.parent_curve.clone(),
                ),
                lock_together,
                is_main_path: true,
            });
            dist = 0.0;
        }
        if lock_together {
            for lock in other_locks.unwrap().path_slices_e.iter() {
                let lock = path_slices_query.get(*lock);
                if lock.is_err() {
                    continue;
                }
                let (lock, _) = lock.unwrap();
                intent.path_locks.push_back(PathSliceLock {
                    path_slice: lock.clone(),
                    lock_together,
                    is_main_path: false,
                });
            }
        }
    }
}

fn remove_car_path(
    car: &mut Mut<Car>,
    path_slice: &PathSlice,
    path_slice_query: &mut Query<&mut PathSlice>,
) {
    let mut pop_e = HashSet::<Entity>::new();
    // println!("paths: {:?} ", car.path_slices);
    for (i, car_ps_e) in car.path_slices.iter_mut().enumerate() {
        let mut car_ps = path_slice_query.get_mut(*car_ps_e).unwrap();
        if car_ps.path_e != path_slice.path_e {
            continue;
        }
        if car_ps.end <= path_slice.end {
            pop_e.insert(*car_ps_e);
            // println!("path_slice: removed: {:?} ", car_ps);
        } else if car_ps.start < path_slice.end {
            car_ps.start = path_slice.end;
            // println!("path_slice: trimmed: {:?} ", car_ps);
        } else {
            // nothing
        }
    }

    car.path_slices.retain(|e| !pop_e.contains(e));
}

const LOCKED_INTERVAL: f32 = 10.5;

fn digest_approved_intent(
    car: &mut Mut<Car>,
    intent: &mut Mut<PathIntent>,
    mut lock: Mut<PathSlicesLocked>,
    mut path_slice_query: Query<&mut PathSlice>,
) {
    // digest next 1s approved intent, and then update insert into PathSlicesLocked
    let mut dist = 1.0_f32.max(car.speed * LOCKED_INTERVAL * 1.0)
    // .max( 0.5 * car.acceleration * LOCKED_INTERVAL * LOCKED_INTERVAL + car.speed * LOCKED_INTERVAL) 
    + car.length + 1.0;
    while dist > 0.0
        || (intent.path_locks.len() > 0 && intent.path_locks.get(0).unwrap().lock_together)
    {
        if intent.path_locks.len() == 0 {
            break;
        }
        let path_lock = &mut intent.path_locks.get_mut(0).unwrap();
        let path_slice = &mut path_lock.path_slice;
        if path_slice.length() <= dist || path_lock.lock_together {
            let path_lock = intent.path_locks.pop_front().unwrap();
            if path_lock.is_main_path {
                dist -= path_lock.path_slice.length();
                remove_car_path(car, &path_lock.path_slice, &mut path_slice_query);
            }

            lock.locks.push_back(path_lock);
            continue;
        } else {
            let new_start = path_slice.parent_t_of_length(dist);
            assert!(new_start > path_slice.start);
            assert!(new_start < path_slice.end);
            let new_path_slice = PathSlice::new(
                path_slice.path_e,
                path_slice.start,
                new_start,
                path_slice.parent_curve.clone(),
            );
            remove_car_path(car, &new_path_slice, &mut path_slice_query);
            lock.locks.push_back(PathSliceLock {
                path_slice: new_path_slice,
                is_main_path: path_lock.is_main_path,
                lock_together: false,
            });

            path_slice.start = new_start;
            break;
        }
    }
}

pub fn car_intents_lock(
    mut commands: Commands,
    mut intent_query: Query<(
        Entity,
        &mut Car,
        &mut PathIntent,
        &mut PathSlicesLocked,
        Option<&mut PathIntentApproved>,
    )>,
    mut path_slice_query: Query<(&mut PathSlice, Option<&PathLockTogether>)>,
) {
    for (e, mut car, mut intent, lock, approved) in intent_query.iter_mut() {
        // digest approved intent_query
        if approved.is_some() {
            // println!("!!! approved intent_query: {:?}", e);
            commands.entity(e).remove::<PathIntentApproved>();
            digest_approved_intent(
                &mut car,
                &mut intent,
                lock,
                path_slice_query.transmute_lens::<&mut PathSlice>().query(),
            );
        } else {
            // println!("!!! not approved intent_query: {:?}", e);
        }
    }
}

pub fn car_intent_update(
    time: Res<Time>,
    mut cars: Query<(&Car, &mut PathIntent)>,
    mut path_slices_query: Query<(&mut PathSlice, Option<&PathLockTogether>)>,
) {
    let now = time.elapsed_seconds();
    for (car, mut intent) in cars.iter_mut() {
        // TODO: update more frequently when locked length is not enough
        if now - intent.last_update < 0.25 {
            continue;
        }
        intent.last_update = now;
        update_one_car_intent(&car, &mut intent, &mut path_slices_query);
    }
}

fn adjust_car_acceleration(car: &mut Mut<Car>, locks: &Mut<PathSlicesLocked>) {
    let mut remain_dist = (locks
        .locks
        .iter()
        .filter_map(|lock| {
            if lock.is_main_path {
                Some(lock.path_slice.length())
            } else {
                None
            }
        })
        .sum::<f32>())
    .max(0.0);
    if remain_dist < car.length {
        car.acceleration = 0.0;
        car.speed = 0.0;
        return;
    }

    remain_dist -= car.length;

    if remain_dist > car.speed * 1.0 {
        // 前面很空就油門踩死
        // accelerate to max if the front is clear
        car.acceleration = car.acc_max;
        return;
    }

    let mut locked_interval_adj: f32 = 1.0;
    if remain_dist > 1.0 && car.speed < 2.0 {
        locked_interval_adj *= 0.5;
    }
    // 100kph / 2s = 27.8m/s / 2s = 13.9m/s^2
    let mut acc = 2.0 * (remain_dist - car.speed * locked_interval_adj)
        / (locked_interval_adj * locked_interval_adj);

    acc = acc.min(car.acc_max);

    // v0t + 0.5at^2 = s
    // => a = 2(s - v0t) / t^2
    car.acceleration = acc;
}

pub fn car_move(
    mut car_query: Query<(Entity, &mut Car, &mut Transform)>,
    mut locked_path_slices_query: Query<&mut PathSlicesLocked>,

    time: Res<Time>,
) {
    for (car_e, mut car, mut transform) in car_query.iter_mut() {
        let locked_path_slices = locked_path_slices_query.get_mut(car_e);
        if locked_path_slices.is_err() {
            continue;
        }
        let mut locked_path_slices = locked_path_slices.unwrap();
        adjust_car_acceleration(&mut car, &locked_path_slices);
        car.speed += car.acceleration * time.delta_seconds();
        car.speed = car.speed.min(30.0).max(0.0);

        let mut distance = car.speed * time.delta_seconds();
        let mut position = car.last_position;

        let mut idx_offset = 0;
        while distance > 0.0 {
            if locked_path_slices.locks.len() <= idx_offset {
                println!("!!! no more locked path slices!!!");
                break;
            }
            let lock = &mut locked_path_slices.locks.get_mut(idx_offset).unwrap();
            if !lock.is_main_path {
                idx_offset += 1;
                continue;
            }
            // println!("!!! [move] lock of path_slice: {:?}", lock.path_slice);
            let path_slice = &mut lock.path_slice;
            if path_slice.length() <= distance {
                // println!("!!! [move] lock of path_slice removed: {:?}", path_slice);
                let path_slice = locked_path_slices.locks.remove(idx_offset).unwrap().path_slice;
                position = path_slice.position(1.0);
                distance -= path_slice.length();
            } else {
                position = path_slice.position(path_slice.t_of_length(distance));
                path_slice.start = path_slice.parent_t_of_length(distance);
                break;
            }
        }
        if locked_path_slices.locks.len() <= idx_offset
            || !locked_path_slices.locks[idx_offset].lock_together
        {
            locked_path_slices.locks = locked_path_slices.locks.clone().split_off(idx_offset);
        }

        if position == car.last_position {
            continue;
        }

        let translate = Transform::from_translation(car.last_position);
        let rotation = translate.looking_at(position, Vec3::Y);
        car.last_position = position;
        // align the car's front/center/bottom with the path
        let shift = Transform::from_translation(Vec3::new(0.0, 0.5, -0.8));
        *transform = rotation * shift;
    }
}

pub fn test_setup_car_and_path(
    mut commands: Commands,
    mut lock_index: ResMut<PathLockIndex>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let curve_a = QuadraticBezierCurve::new([
        Vec3::new(-9., 0.0, 9.0),
        Vec3::new(0.0, 0.0, 9.0),
        Vec3::new(0.0, 0.0, 6.0),
    ])
    .to_curve();

    let path_a = commands
        .spawn(Path {
            curve: curve_a.clone(),
            left: None,
            right: None,
        })
        .id();

    let curve_c = QuadraticBezierCurve::new([
        Vec3::new(9., 0.0, 9.0),
        Vec3::new(0.0, 0.0, 9.0),
        Vec3::new(0.0, 0.0, 6.0),
    ])
    .to_curve();
    let path_c = commands
        .spawn(Path {
            curve: curve_c.clone(),
            left: None,
            right: None,
        })
        .id();
    let curve_e = QuadraticBezierCurve::new([
        Vec3::new(0.0, 0.0, 6.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -29.0),
    ])
    .to_curve();
    let path_e = commands
        .spawn(Path {
            curve: curve_e.clone(),
            left: None,
            right: None,
        })
        .id();

    link_next(&mut commands, path_a, 1.0, path_e, 0.0);
    link_next(&mut commands, path_c, 1.0, path_e, 0.0);

    for i in 0..4 {
        let slice_a = PathSlice::new(path_a, 0.0, 0.85, curve_a.clone());
        let slice_b = PathSlice::new(path_a, 0.85, 1.0, curve_a.clone());

        let slice_c = PathSlice::new(path_c, 0.0, 0.85, curve_c.clone());
        let slice_d = PathSlice::new(path_c, 0.85, 1.0, curve_c.clone());

        let slice_e = PathSlice::new(path_e, 0.0, 0.02, curve_e.clone());
        let slice_f = PathSlice::new(path_e, 0.02, 1.0, curve_e.clone());
        let mut spawn = |s| commands.spawn(s).id();
        let car_a_slices = VecDeque::from(vec![
            spawn(slice_a.clone()),
            spawn(slice_b.clone()),
            spawn(slice_e.clone()),
            spawn(slice_f.clone()),
        ]);

        let car_b_slices = VecDeque::from(vec![
            spawn(slice_c.clone()),
            spawn(slice_d.clone()),
            spawn(slice_e.clone()),
            spawn(slice_f.clone()),
        ]);

        let lock_group_a = vec![
            spawn(slice_b.clone()),
            spawn(slice_d.clone()),
            spawn(slice_e.clone()),
        ];
        let lock_group_b = vec![
            spawn(slice_b.clone()),
            spawn(slice_d.clone()),
            spawn(slice_e.clone()),
        ];
        let lock_group_c = vec![
            spawn(slice_b.clone()),
            spawn(slice_d.clone()),
            spawn(slice_e.clone()),
        ];
        let lock_group_d = vec![
            spawn(slice_b.clone()),
            spawn(slice_d.clone()),
            spawn(slice_e.clone()),
        ];

        commands.entity(car_a_slices[1]).insert(PathLockTogether {
            path_slices_e: lock_group_a.into(),
        });
        commands.entity(car_a_slices[2]).insert(PathLockTogether {
            path_slices_e: lock_group_b.into(),
        });

        commands.entity(car_b_slices[1]).insert(PathLockTogether {
            path_slices_e: lock_group_c.into(),
        });
        commands.entity(car_b_slices[2]).insert(PathLockTogether {
            path_slices_e: lock_group_d.into(),
        });

        // car on path_a
        let car_a_e = commands
            .spawn(CarBundle {
                // a cube
                pbr: PbrBundle {
                    mesh: meshes.add(Cuboid::new(1.0, 1.0, 2.0)),
                    material: materials.add(Color::WHITE),

                    transform: Transform::from_translation(Vec3::new(-9.0, 1.0, 999.0)),
                    ..Default::default()
                },
                car: Car {
                    length: 2.1,
                    speed: 0.0,
                    acceleration: 0.0,
                    acc_max: 23.9 + rand::random::<f32>() * 5.0,
                    path_slices: car_a_slices.clone(),
                    last_position: Vec3::ONE * 999.0,
                },
                intent: PathIntent::empty(),
                locks: PathSlicesLocked::empty(),
            })
            .id();

        for path_slice_e in car_a_slices.iter() {
            commands.entity(car_a_e).add_child(*path_slice_e);
            commands.entity(*path_slice_e).set_parent(car_a_e);
        }
        // car on path_c
        let car_b_e = commands
            .spawn(CarBundle {
                // a cube
                pbr: PbrBundle {
                    mesh: meshes.add(Cuboid::new(1.0, 1.0, 2.0)),
                    material: materials.add(Color::WHITE),

                    transform: Transform::from_xyz(999.0, 1.0, 0.0),
                    ..Default::default()
                },
                car: Car {
                    length: 2.1,
                    speed: 0.,
                    acceleration: 0.0,
                    acc_max: 23.9 + rand::random::<f32>() * 5.0,
                    path_slices: car_b_slices.clone(),
                    last_position: Vec3::ONE * 999.,
                },
                intent: PathIntent::empty(),
                locks: PathSlicesLocked::empty(),
            })
            .id();

        for path_slice_e in car_b_slices.iter() {
            commands.entity(car_b_e).add_child(*path_slice_e);
            commands.entity(*path_slice_e).set_parent(car_b_e);
        }

        lock_index.locked.insert(car_a_e);
        lock_index.locked.insert(car_b_e);
    }
}
