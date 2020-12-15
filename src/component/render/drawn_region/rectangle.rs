use std::fmt::Debug;

use super::*;

/// Represents an unrotated rectangular drawn region. This is one of the simplest
/// implementations of `DrawnRegion`.
#[derive(Clone, Copy, Debug)]
pub struct RectangularDrawnRegion {
    left: f32,
    bottom: f32,
    right: f32,
    top: f32,
}

impl RectangularDrawnRegion {
    /// Constructs a new `RectangularDrawnRegion` with the given left bound, bottom
    /// bound, right bound and top bound.
    pub fn new(left: f32, bottom: f32, right: f32, top: f32) -> Self {
        Self {
            left,
            bottom,
            right,
            top,
        }
    }
}

impl DrawnRegion for RectangularDrawnRegion {
    fn is_inside(&self, x: f32, y: f32) -> bool {
        x >= self.left && x <= self.right && y >= self.bottom && y <= self.top
    }

    fn clone(&self) -> Box<dyn DrawnRegion> {
        Box::new(*self)
    }

    fn get_left(&self) -> f32 {
        self.left
    }

    fn get_bottom(&self) -> f32 {
        self.bottom
    }

    fn get_right(&self) -> f32 {
        self.right
    }

    fn get_top(&self) -> f32 {
        self.top
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_is_inside() {
        let rect = RectangularDrawnRegion::new(-0.2, -0.4, 0.6, 1.0);

        // Boundary cases
        assert!(rect.is_inside(-0.2, -0.4));
        assert!(rect.is_inside(0.6, 1.0));
        assert!(!rect.is_inside(-0.21, 0.0));
        assert!(!rect.is_inside(0.0, 1.01));

        // Simpler cases
        assert!(rect.is_inside(0.0, 0.0));
        assert!(!rect.is_inside(2.0, -3.5));
    }

    #[test]
    fn test_invalid() {
        let rect = RectangularDrawnRegion::new(1.0, 1.0, -1.0, -1.0);

        assert!(!rect.is_inside(0.0, 0.0));
        assert!(!rect.is_inside(1.0, 1.0));
    }

    #[test]
    fn test_bounds() {
        let rect = RectangularDrawnRegion::new(0.3, 0.8, 1.0, 1.5);

        assert_eq!(0.3, rect.get_left());
        assert_eq!(0.8, rect.get_bottom());
        assert_eq!(1.0, rect.get_right());
        assert_eq!(1.5, rect.get_top());
    }
}
