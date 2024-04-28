mod curve_column;

// pub use curve_column::CurveColumn;
pub trait Collider {
    fn is_collision(&self, other: &Self) -> bool;
}
pub trait IndexedCollider: Collider {
    // x0, y0, x1, y1,  x1 > x0 and y1 > y0
    fn key() -> (f32, f32, f32, f32);
}
