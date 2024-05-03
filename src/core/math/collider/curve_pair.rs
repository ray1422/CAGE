trait Curve {}
trait Line {
    fn intersection(&self);
}

trait Intersection<T> {
    fn intersection(&self, s: T) -> bool;
}

impl Intersection<&dyn Line> for dyn Line {
    fn intersection(&self, s: &dyn Line) -> bool {
        todo!()
    }
}
