use std::{borrow::BorrowMut, collections::VecDeque};

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use cage::core::math::curve::Curve;
use rand::prelude::*;

/// PathSlice is a slice of a path
/// used in intent and lock operations
#[derive(Component, Debug, Clone)]
pub struct PathSlice {
    pub path_e: Entity,
    pub start: f32,
    pub end: f32,
    pub parent_curve: Curve,
}

impl PathSlice {
    pub fn new(path_e: Entity, start: f32, end: f32, parent_curve: Curve) -> Self {
        Self {
            path_e,
            start,
            end,
            parent_curve,
        }
    }

    pub fn length(&self) -> f32 {
        self.parent_curve.length() * (self.end - self.start)
    }

    pub fn parent_t_of_length(&self, length: f32) -> f32 {
        self.start + length / self.length() * (self.end - self.start)
    }

    pub fn t_of_length(&self, length: f32) -> f32 {
        length / (self.length())
    }

    pub fn position(&self, progress: f32) -> Vec3 {
        self.parent_curve
            .position(self.start + progress * (self.end - self.start))
    }
}

impl PartialEq for PathSlice {
    fn eq(&self, other: &Self) -> bool {
        self.path_e == other.path_e
            && (self.start - other.start).abs() < 1e-6
            && (self.end - other.end).abs() < 1e-6
    }
}

#[derive(Component, Debug, Clone)]
pub struct PathLockTogether {
    pub path_slices_e: VecDeque<Entity>,
}
#[derive(Component, Debug, Clone)]
pub struct PathSliceLock {
    pub path_slice: PathSlice,
    pub lock_together: bool,
    pub is_main_path: bool,
}

#[derive(Debug, Clone, Component)]
pub struct PathIntent {
    pub path_locks: VecDeque<PathSliceLock>,
    /// if the paths can be trimmed from the end
    pub priority: i16,
    /// ms when the intent is available
    pub est_available_at: u64,
    pub last_update: f32,
}

impl PathIntent {
    pub fn empty() -> Self {
        Self {
            path_locks: VecDeque::new(),
            priority: 0,
            est_available_at: 0,
            last_update: 0.0,
        }
    }
}

#[derive(Debug, Clone, Component)]
/// PathIntentApproved is a marker component for approved intents. driver can remove this and add Locked
pub struct PathIntentApproved {}

#[derive(Debug, Clone, Component)]
pub struct PathSlicesLocked {
    pub locks: VecDeque<PathSliceLock>,
    pub est_available_at: u64,
}

impl PathSlicesLocked {
    pub fn empty() -> Self {
        Self {
            locks: VecDeque::new(),
            est_available_at: 0,
        }
    }
}

#[derive(Debug, Clone, Resource)]
pub struct PathLockIndex {
    /// locked: path_e => (lock_e, idx of lock, lock)
    index: HashMap<Entity, HashMap<Entity, Vec<(usize, PathSliceLock)>>>,
    /// reverse_index: lock_e => Set(path_e)
    reverse_index: HashMap<Entity, HashSet<Entity>>,
}

impl PathLockIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            reverse_index: HashMap::new(),
        }
    }

    pub fn upsert_locks(&mut self, lock_e: &Entity, locks: impl Iterator<Item = PathSliceLock>) {
        // remove old index
        let path_es = self.reverse_index.remove(lock_e);
        path_es
            .into_iter()
            .flat_map(|e| e.into_iter())
            .for_each(|path_e| {
                let path_mp = self.index.get_mut(&path_e).unwrap();
                path_mp.remove(lock_e);
            });
        for (i, lock) in locks.enumerate() {
            self.reverse_index
                .entry(*lock_e)
                .or_default()
                .insert(lock.path_slice.path_e);
            self.index
                .entry(lock.path_slice.path_e)
                .or_insert(HashMap::new())
                .entry(*lock_e)
                .or_insert(Vec::new())
                .push((i, lock));
        }
    }

    pub fn collections(
        &self,
        path_e: &Entity,
        start: f32,
        end: f32,
    ) -> impl Iterator<Item = (Entity, usize, &PathSliceLock)> {
        self.index
            .get(path_e)
            .into_iter()
            .flat_map(|f| f.iter())
            .flat_map(|(lock_e, locks)| locks.into_iter().map(|(i, lock)| (*lock_e, *i, lock)))
            .filter(move |(_, _, lock)| lock.path_slice.start < end && lock.path_slice.end > start)
    }
}

pub fn trim_lock(lock: &mut PathSliceLock, until: f32) -> bool {
    if lock.lock_together {
        return false;
    }
    if lock.path_slice.start >= until {
        return false;
    }
    if lock.path_slice.end <= until {
        // shouldn't happen
        return true;
    }
    lock.path_slice.end = until;
    true
}

fn pop_locks_until_no_group(locks: &mut VecDeque<PathSliceLock>) {
    while let Some(lock) = locks.back() {
        if lock.lock_together {
            locks.pop_back();
        } else {
            break;
        }
    }
}

pub fn schedule_intents(
    mut commands: Commands,
    mut lock_index: ResMut<PathLockIndex>, // TODO: use this index the entities
    mut intents: Query<(Entity, &mut PathIntent), Without<PathIntentApproved>>,
    locked_path_slices: Query<(Entity, &PathSlicesLocked)>,
) {
    let mut pending_intents: VecDeque<(Entity, PathIntent)> = VecDeque::new();

    for (intent_e, mut intent) in intents.iter_mut() {
        'l: loop {
            for (j, path_lock) in intent.path_locks.iter_mut().enumerate() {
                for (other_e, _, locked_path_slice) in lock_index.collections(
                    &path_lock.path_slice.path_e,
                    path_lock.path_slice.start,
                    path_lock.path_slice.end,
                ) {
                    if other_e == intent_e {
                        continue;
                    }

                    if path_lock.path_slice.path_e != locked_path_slice.path_slice.path_e {
                        continue;
                    }
                    if path_lock.path_slice.start < locked_path_slice.path_slice.end
                        && path_lock.path_slice.end > locked_path_slice.path_slice.start
                    {
                        // continue 'm;
                        if trim_lock(path_lock, locked_path_slice.path_slice.start) {
                            intent.path_locks.truncate(j + 1);
                        } else {
                            intent.path_locks.truncate(j);
                            pop_locks_until_no_group(&mut intent.path_locks);
                        }
                        continue 'l;
                    }
                }
            }
            break;
        }

        if intent.path_locks.is_empty() {
            continue;
        }
        let mut other_iter = pending_intents.iter_mut();
        let intent_priority = intent.priority;
        'l: loop {
            for (other_e, other_intent) in other_iter.by_ref() {
                let other_priority = other_intent.priority;
                'self_lock: for (i, lock) in intent.path_locks.iter_mut().enumerate() {
                    'other_locks: for (j, other_lock) in
                        other_intent.path_locks.iter_mut().enumerate()
                    {
                        let ps = &lock.path_slice;
                        let other_ps = &other_lock.path_slice;
                        if ps.path_e != other_ps.path_e {
                            continue;
                        }
                        if !(ps.start < other_ps.end && ps.end > other_ps.start) {
                            // no overlap
                            continue;
                        }
                        let mut tmp_other_priority = other_priority;
                        // overlap
                        if tmp_other_priority == intent_priority {
                            // other_priority + or - 1 randomly
                            tmp_other_priority += if rand::random() { 1 } else { -1 };
                        }
                        if tmp_other_priority > intent_priority {
                            // other has higher priority
                            if trim_lock(lock, other_ps.start) {
                                intent.path_locks.truncate(i + 1);
                            } else {
                                intent.path_locks.truncate(i);
                                pop_locks_until_no_group(&mut intent.path_locks);
                            }
                            continue 'l;
                        } else {
                            if trim_lock(other_lock, ps.start) {
                                other_intent.path_locks.truncate(j + 1);
                            } else {
                                other_intent.path_locks.truncate(j);
                                pop_locks_until_no_group(&mut other_intent.path_locks);
                            }
                            continue 'l;
                        }
                    }
                }
            }
            break;
        }
        pending_intents.push_back((intent_e, intent.clone()));
    }

    for (intent_e, intent) in pending_intents {
        // println!("approved intent {:?} {:#?}", intent_e, intent);
        commands.entity(intent_e).insert(intent);
        commands.entity(intent_e).insert(PathIntentApproved {});
    }
}
