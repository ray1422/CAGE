use bevy::{prelude::*, utils::HashSet};
use cage::core::math::curve::Curve;

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
        length / self.length()
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
    pub path_slices_e: Vec<Entity>,
}
#[derive(Component, Debug, Clone)]
pub struct PathSliceLock {
    pub path_slice: PathSlice,
    pub lock_together: bool,
    pub is_main_path: bool,
}

#[derive(Debug, Clone, Component)]
pub struct PathIntent {
    pub path_locks: Vec<PathSliceLock>,
    /// if the paths can be trimmed from the end
    pub priority: i16,
    /// ms when the intent is available
    pub est_available_at: u64,
}

impl PathIntent {
    pub fn empty() -> Self {
        Self {
            path_locks: Vec::new(),

            priority: 0,
            est_available_at: 0,
        }
    }
}

#[derive(Debug, Clone, Component)]
/// PathIntentApproved is a marker component for approved intents. driver can remove this and add Locked
pub struct PathIntentApproved {}

#[derive(Debug, Clone, Component)]
pub struct PathSlicesLocked {
    pub locks: Vec<PathSliceLock>,
    pub est_available_at: u64,
}

impl PathSlicesLocked {
    pub fn empty() -> Self {
        Self {
            locks: Vec::new(),
            est_available_at: 0,
        }
    }
}

#[derive(Debug, Clone, Resource)]
pub struct PathLockIndex {
    pub intents: HashSet<Entity>,
    pub locked: HashSet<Entity>,
}

impl PathLockIndex {
    pub fn new() -> Self {
        Self {
            intents: HashSet::new(),
            locked: HashSet::new(),
        }
    }
}

pub fn schedule_intents(
    mut commands: Commands,
    mut lock_index: ResMut<PathLockIndex>,
    mut intents: Query<(Entity, &mut PathIntent), Without<PathIntentApproved>>,
    approved_intents: Query<(Entity, &mut PathIntent), With<PathIntentApproved>>,
    // approved intents can't be changed
    locked_path_slices: Query<(Entity, &PathSlicesLocked)>,
) {
    let mut pending_intents: Vec<(Entity, PathIntent)> = vec![];
    let mut pop_set_pending = HashSet::<Entity>::new();
    for (intent_e, mut intent) in intents.iter_mut() {
        // // FIXME: test code
        // commands.get_entity(intent_e).and_then(|mut e| {
        //     e.insert(PathIntentApproved {});
        //     Some(e)
        // });
        // continue;
        let mut can_approve = true;
        for lock in locked_path_slices.iter() {
            if lock.0 == intent_e {
                continue;
            }
            for path_lock in intent.path_locks.iter() {
                for locked_path_slice in lock.1.locks.iter() {
                    if path_lock.path_slice.path_e == locked_path_slice.path_slice.path_e {
                        if path_lock.path_slice.start < locked_path_slice.path_slice.end
                            && path_lock.path_slice.end > locked_path_slice.path_slice.start
                        {
                            can_approve = false;
                            break;
                        }
                    }
                }
            }
        }
        if !can_approve {
            continue;
        }

        'other_intents: for (other_e, other_intent) in pending_intents.iter() {
            #[allow(unused_labels)]
            'self_locks: for lock in intent.path_locks.iter() {
                'other_locks: for other_lock in other_intent.path_locks.iter() {
                    let ps = &lock.path_slice;
                    let other_ps = &other_lock.path_slice;
                    if ps.path_e != other_ps.path_e {
                        continue;
                    }
                    if !(ps.start < other_ps.end && ps.end > other_ps.start) {
                        // no overlap
                        continue;
                    }
                    // overlap
                    if other_intent.priority > intent.priority {
                        // other has higher priority
                        pop_set_pending.insert(intent_e);
                        break 'other_intents;
                    } else {
                        // this has higher priority
                        pop_set_pending.insert(*other_e);
                        break 'other_locks;
                    }
                }
            }
        }
        pending_intents.push((intent_e, intent.clone()));
        pending_intents.retain(|(e, _)| !pop_set_pending.contains(e));
    }

    for (intent_e, intent) in pending_intents.iter() {
        commands.get_entity(*intent_e).and_then(|mut e| {
            println!("approved intent: {:?}, {:?}", intent_e, intent);
            e.insert(PathIntentApproved {});
            Some(e)
        });
    }
}
