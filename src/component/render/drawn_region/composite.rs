use crate::DrawnRegion;

/// A `DrawnRegion` that is composed of other `DrawnRegion`s (typically more than
/// 1). Points will be considered *inside* a `CompositeDrawnRegion` if it is
/// *inside* at least 1 of the `DrawnRegion`s it is composed of.
pub struct CompositeDrawnRegion {

    components: Vec<Box<dyn DrawnRegion>>,

    left_bound: f32,
    bottom_bound: f32,
    right_bound: f32,
    top_bound: f32
}

impl CompositeDrawnRegion {
    /// Constructs a new `CompositeDrawnRegion` that will be composed of the
    /// given *components*.
    pub fn new(components: Vec<Box<dyn DrawnRegion>>) -> Self {
        let mut left_bound = f32::INFINITY;
        let mut bottom_bound = f32::INFINITY;
        let mut right_bound = -f32::INFINITY;
        let mut top_bound = -f32::INFINITY;

        for component in &components {
            left_bound = f32::min(left_bound, component.get_left());
            bottom_bound = f32::min(bottom_bound, component.get_bottom());
            right_bound = f32::max(right_bound, component.get_right());
            top_bound = f32::max(top_bound, component.get_top());
        }

        Self {
            components,
            left_bound,
            bottom_bound,
            right_bound,
            top_bound
        }
    }
}

impl DrawnRegion for CompositeDrawnRegion {
    fn is_inside(&self, x: f32, y: f32) -> bool {
        for component in &self.components {
            if component.is_within_bounds(x, y) && component.is_inside(x, y) {
                return true;
            }
        }

        false
    }

    fn clone(&self) -> Box<dyn DrawnRegion> {
        let components = self.components.iter().map(
            |component| component.as_ref().clone()
        ).collect();
        Box::new(Self {
            components,
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
    fn test_empty() {
        let empty = CompositeDrawnRegion::new(Vec::new());
        // is_inside and is_within_bounds should always return false
        assert!(!empty.is_inside(0.4, 14.0));
        assert!(!empty.is_within_bounds(0.0, 0.0));
        assert!(!empty.is_inside(-1.0, -2.0));
        assert!(!empty.is_inside(-2.0, 3.0));
    }

    #[test]
    fn test_single() {
        let single = CompositeDrawnRegion::new(vec![
            Box::new(RectangularDrawnRegion::new(0.2, 1.0, 0.5, 2.0))
        ]);

        assert!(!single.is_inside(0.1, 0.9));
        assert!(single.is_inside(0.2, 2.0));

        assert_eq!(0.2, single.get_left());
        assert_eq!(1.0, single.get_bottom());
        assert_eq!(0.5, single.get_right());
        assert_eq!(2.0, single.get_top());
    }

    #[test]
    fn test_double() {
        let double = CompositeDrawnRegion::new(vec![
            Box::new(RectangularDrawnRegion::new(0.0, 0.0, 1.0, 1.0)),
            Box::new(RectangularDrawnRegion::new(2.0, 1.0, 3.0, 2.0))
        ]);

        assert!(double.is_inside(0.1, 0.1));
        assert!(!double.is_inside(1.1, 0.1));
        assert!(!double.is_inside(2.1, 0.1));
        assert!(!double.is_inside(0.1, 1.1));
        assert!(double.is_inside(2.1, 1.1));

        assert!(!double.is_inside(-0.1, 0.1));
        assert!(!double.is_inside(3.1, 1.1));

        assert_eq!(0.0, double.get_left());
        assert_eq!(0.0, double.get_bottom());
        assert_eq!(3.0, double.get_right());
        assert_eq!(2.0, double.get_top());
    }
}