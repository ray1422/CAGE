use bevy::math::Vec3;

pub trait Line {
    fn intersection(&self, other: &impl Line);
}

pub struct LineByPQ {
    p0: Vec3,
    p1: Vec3,
    q0: Vec3,
    q1: Vec3,
}
