use crate::*;

pub struct TransformedDrawnRegion<T: Clone + Fn(f32, f32) -> (f32, f32) + 'static> {
    region: Box<dyn DrawnRegion>,
    transform_function: T,

    left_bound: f32,
    bottom_bound: f32,
    right_bound: f32,
    top_bound: f32
}

impl<T: Clone + Fn(f32, f32) -> (f32, f32) + 'static> TransformedDrawnRegion<T> {
    pub fn new(
        region: Box<dyn DrawnRegion>, transform_function: T,
        left_bound: f32, bottom_bound: f32, right_bound: f32, top_bound: f32
    ) -> Self {
        Self {
            region,
            transform_function,
            left_bound,
            bottom_bound,
            right_bound,
            top_bound
        }
    }
}

impl<T: Clone + Fn(f32, f32) -> (f32, f32) + 'static> DrawnRegion for TransformedDrawnRegion<T> {
    fn is_inside(&self, x: f32, y: f32) -> bool {
        let transformed = (self.transform_function)(x, y);
        println!("Transformed is {:?}", transformed);
        self.region.is_inside(transformed.0, transformed.1)
    }

    fn clone(&self) -> Box<dyn DrawnRegion> {
        Box::new(Self {
            region: self.region.clone(),
            transform_function: self.transform_function.clone(),
            left_bound: self.left_bound,
            bottom_bound: self.bottom_bound,
            right_bound: self.right_bound,
            top_bound: self.top_bound
        })
    }

    fn get_left(&self) -> f32 {
        self.left_bound
    }

    fn get_bottom(&self) -> f32 {
        self.bottom_bound
    }

    fn get_right(&self) -> f32 {
        self.right_bound
    }

    fn get_top(&self) -> f32 {
        self.top_bound
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn basic_test() {
        let original_region = Box::new(RectangularDrawnRegion::new(1.0, 4.0, 2.0, 7.0));
        let region = TransformedDrawnRegion::new(
            original_region, |x, y| (x * 3.0, y - 1.0), 
            1.0 / 3.0, 5.0, 2.0 / 3.0, 8.0
        );
        assert!(!region.is_inside(0.3, 4.5));
        assert!(!region.is_inside(0.4, 4.5));
        assert!(region.is_inside(0.4, 5.5));
        assert!(region.is_inside(0.65, 7.5));
        assert!(!region.is_inside(0.7, 7.5));
        assert!(!region.is_inside(0.7, 8.5));
    }
}