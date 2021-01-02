/// Represents an immutable 2-dimensional point with floating point coordinates.
///
/// In the coordinate system used by this crate, the point `(0.0, 0.0)` indicates the bottom-left
/// corner of a `Component` or `Application` and the point `(1.0, 1.0)` indicates the top-right
/// corner.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    x: f32,
    y: f32,
}

impl Point {
    /// Constructs and returns the point `(x, y)`
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Gets the `x`-coordinate of this point.
    ///
    /// A value of 0.0 indicates the left bound of a `Component` and a value of 1.0 indicates the
    /// right bound.
    pub fn get_x(&self) -> f32 {
        self.x
    }

    /// Gets the `y`-coordinate of this point.
    ///
    /// A value of 0.0 indicates the bottom bound of a `Component` and a value of 1.0 indicates the
    /// top bound.
    pub fn get_y(&self) -> f32 {
        self.y
    }

    /// Computes and returns the (Euclidean) distance from this point to the `other` point
    pub fn distance_to(&self, other: Point) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        f32::sqrt(dx * dx + dy * dy)
    }

    /// Tests if this point is 'nearly' equal to the other point. This is convenient for unit tests
    /// because floating point numbers can have rounding errors.
    ///
    /// Currently, two points are considered nearly equal if their distance is smaller than 0.01
    pub fn nearly_equal(&self, other: Point) -> bool {
        self.distance_to(other) < 0.01
    }
}

#[cfg(test)]
mod tests {

    use super::Point;

    #[test]
    fn test_distance() {
        let x1 = 12.0;
        let y1 = -9.5;
        let x2 = x1 + 3.0;
        let y2 = y1 - 4.0;
        let point1 = Point::new(x1, y1);
        let point2 = Point::new(x2, y2);
        let distance = point1.distance_to(point2);

        // This should be true, even if rounding errors are made
        assert_eq!(distance, point2.distance_to(point1));

        // Some rounding errors are possible
        assert!(distance > 4.99 && distance < 5.01);
    }

    #[test]
    fn test_nearly_equal() {
        assert!(Point::new(10.0, 20.0).nearly_equal(Point::new(10.0001, 19.999)));
        assert!(!Point::new(10.0, 20.0).nearly_equal(Point::new(10.1, 19.9)));
        assert!(Point::new(-10.0, 20.0).nearly_equal(Point::new(-10.0001, 19.999)));
        assert!(!Point::new(-10.0, 20.0).nearly_equal(Point::new(-10.1, 19.9)));
    }
}
