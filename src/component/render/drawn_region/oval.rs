use std::fmt::Debug;

use crate::*;

/// Represents an oval drawn region (like a circle, but the radius on the x-axis might not be the
/// same as the radius on the y-axis). This is a quite simple implementation of `DrawnRegion`.
#[derive(Copy, Clone, Debug)]
pub struct OvalDrawnRegion {
    center: Point,
    radius_x: f32,
    radius_y: f32,
}

impl OvalDrawnRegion {
    /// Constructs a new `OvalDrawnRegion` with the given *center* `Point`, radius on the x-axis,
    /// and radius on the y-axis.
    ///
    /// # Boundary points
    /// The point (center.x - radius_x, center.y) is the left-most point that lies on this oval. The
    /// point (center.x, center.y - radius) is the lowest point that lies on this oval. The point
    /// (center.x + radius_x, center.y) is the right-most point that lies on this oval. The point
    /// (center.x, center.y + radius) is the highest point that lies on this oval.
    pub fn new(center: Point, radius_x: f32, radius_y: f32) -> Self {
        Self { center, radius_x, radius_y }
    }
}

impl DrawnRegion for OvalDrawnRegion {
    fn is_inside(&self, point: Point) -> bool {
        let dx = (self.center.get_x() - point.get_x()) / self.radius_x;
        let dy = (self.center.get_y() - point.get_y()) / self.radius_y;
        return dx * dx + dy * dy <= 1.0;
    }

    fn clone(&self) -> Box<dyn DrawnRegion> {
        Box::new(*self)
    }

    fn get_left(&self) -> f32 {
        self.center.get_x() - self.radius_x
    }

    fn get_bottom(&self) -> f32 {
        self.center.get_y() - self.radius_y
    }

    fn get_right(&self) -> f32 {
        self.center.get_x() + self.radius_x
    }

    fn get_top(&self) -> f32 {
        self.center.get_y() + self.radius_y
    }

    fn find_line_intersection(&self, from: Point, to: Point) -> LineIntersection {
        let distance = from.distance_to(to);
        let direction_x = (to.get_x() - from.get_x()) / distance;
        let direction_y = (to.get_y() - from.get_y()) / distance;

        let from_inside = self.is_inside(from);
        let to_inside = self.is_inside(to);
        match (from_inside, to_inside) {
            (true, true) => LineIntersection::FullyInside,
            (true, false) => {
                // TODO Line leaves the oval
                unimplemented!()
            },
            (false, true) => {
                // TODO Line enters the oval
                unimplemented!()
            },
            (false, false) => {
                // TODO Determine whether or not there is an intersection
                unimplemented!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_bounds() {
        let oval = OvalDrawnRegion::new(Point::new(0.5, -0.5), 2.5, 0.5);

        // The numbers are chosen such that no rounding errors should occur
        assert_eq!(-2.0, oval.get_left());
        assert_eq!(-1.0, oval.get_bottom());
        assert_eq!(3.0, oval.get_right());
        assert_eq!(0.0, oval.get_top());
    }

    #[test]
    fn test_is_inside() {
        let oval = OvalDrawnRegion::new(Point::new(5.0, 3.0), 3.0, 0.5);

        // The numbers are chosen such that no rounding errors should occur

        // Obviously, the center should be inside
        assert!(oval.is_inside(Point::new(5.0, 3.0)));

        // As well as the points close to the center
        assert!(oval.is_inside(Point::new(5.1, 3.2)));
        assert!(oval.is_inside(Point::new(4.8, 3.0)));

        // The horizontal edge cases
        assert!(oval.is_inside(Point::new(2.0, 3.0)));
        assert!(!oval.is_inside(Point::new(1.9, 3.0)));
        assert!(!oval.is_inside(Point::new(2.0, 3.1)));
        assert!(oval.is_inside(Point::new(8.0, 3.0)));
        assert!(!oval.is_inside(Point::new(8.1, 3.0)));
        assert!(!oval.is_inside(Point::new(8.0, 2.9)));

        // The vertical edge cases
        assert!(oval.is_inside(Point::new(5.0, 2.5)));
        assert!(!oval.is_inside(Point::new(5.0, 2.4)));
        assert!(!oval.is_inside(Point::new(4.9, 2.5)));
        assert!(oval.is_inside(Point::new(5.0, 3.5)));
        assert!(!oval.is_inside(Point::new(5.0, 3.6)));
        assert!(!oval.is_inside(Point::new(5.1, 3.5)));

        // Now the points at a 45 degree angle
        let factor = 0.5 * 2.0f32.sqrt();

        // Bottom left
        assert!(oval.is_inside(Point::new(5.0 - 2.95 * factor, 3.0 - 0.45 * factor)));
        assert!(!oval.is_inside(Point::new(5.0 - 3.05 * factor, 3.0 - 0.55 * factor)));

        // Bottom right
        assert!(oval.is_inside(Point::new(5.0 + 2.95 * factor, 3.0 - 0.45 * factor)));
        assert!(!oval.is_inside(Point::new(5.0 + 3.05 * factor, 3.0 - 0.55 * factor)));

        // Top left
        assert!(oval.is_inside(Point::new(5.0 - 2.95 * factor, 3.0 + 0.45 * factor)));
        assert!(!oval.is_inside(Point::new(5.0 - 3.05 * factor, 3.0 + 0.55 * factor)));

        // Top right
        assert!(oval.is_inside(Point::new(5.0 + 2.95 * factor, 3.0 + 0.45 * factor)));
        assert!(!oval.is_inside(Point::new(5.0 + 3.05 * factor, 3.0 + 0.55 * factor)));

        // These points are not even close
        assert!(!oval.is_inside(Point::new(0.0, 0.0)));
        assert!(!oval.is_inside(Point::new(-30.0, -5.0)));
        assert!(!oval.is_inside(Point::new(100.0, 50.0)));
    }

    #[test]
    fn test_find_line_intersection() {

        // Let's use a reasonably simple oval for testing
        let oval = OvalDrawnRegion::new(Point::new(10.0, 5.0), 4.0, 3.0);
        // The oval spans the area { min_x: 6.0, min_y: 2.0, max_x: 14.0, max_y: 8.0 }

        // Test fully inside
        assert_eq!(LineIntersection::FullyInside, oval.find_line_intersection(
            Point::new(9.0, 3.5), Point::new(10.0, 6.0)
        ));
        assert_eq!(LineIntersection::FullyInside, oval.find_line_intersection(
            Point::new(10.0, 5.0), Point::new(11.0, 7.0)
        ));
        assert_eq!(LineIntersection::FullyInside, oval.find_line_intersection(
            Point::new(10.0, 2.1), Point::new(10.0, 7.9)
        ));
        assert_eq!(LineIntersection::FullyInside, oval.find_line_intersection(
            Point::new(13.9, 5.0), Point::new(6.1, 5.0)
        ));

        // Test fully outside
        assert_eq!(LineIntersection::FullyOutside, oval.find_line_intersection(
            Point::new(0.0, 5.0), Point::new(5.5, 5.0)
        ));
        assert_eq!(LineIntersection::FullyOutside, oval.find_line_intersection(
            Point::new(10.0, 20.0), Point::new(10.0, 14.3)
        ));
        assert_eq!(LineIntersection::FullyOutside, oval.find_line_intersection(
            Point::new(5.9, 2.0), Point::new(5.9, 20.0)
        ));
        assert_eq!(LineIntersection::FullyOutside, oval.find_line_intersection(
            Point::new(6.1, 2.0), Point::new(6.1, 4.0)
        ));

        // This is root(3) / 2, which is an important constant for circles and ovals
        let hsq3 = 0.5 * 3.0f32.sqrt();

        // A horizontal line that goes 1.5 units above the center
        assert!(LineIntersection::Crosses {
            entrance: Point::new(10.0 - 4.0 * hsq3, 6.5),
            exit: Point::new(10.0 + 4.0 * hsq3, 6.5)
        }.nearly_equal(oval.find_line_intersection(
            Point::new(0.0, 6.5), Point::new(20.0, 6.5)
        )));
        // Variations of that line that intersect the oval only once
        assert!(LineIntersection::Exits {
            point: Point::new(10.0 + 4.0 * hsq3, 6.5)
        }.nearly_equal(oval.find_line_intersection(
            Point::new(8.0, 6.5), Point::new(20.0, 6.5)
        )));
        assert!(LineIntersection::Enters {
            point: Point::new(10.0 - 4.0 * hsq3, 6.5),
        }.nearly_equal(oval.find_line_intersection(
            Point::new(0.0, 6.5), Point::new(12.0, 6.5)
        )));

        // The last 3 lines, but in the other direction
        // A horizontal line that goes 1.5 units above the center
        assert!(LineIntersection::Crosses {
            exit: Point::new(10.0 - 4.0 * hsq3, 6.5),
            entrance: Point::new(10.0 + 4.0 * hsq3, 6.5)
        }.nearly_equal(oval.find_line_intersection(
            Point::new(20.0, 6.5), Point::new(0.0, 6.5)
        )));
        // Variations of that line that intersect the oval only once
        assert!(LineIntersection::Enters {
            point: Point::new(10.0 + 4.0 * hsq3, 6.5)
        }.nearly_equal(oval.find_line_intersection(
            Point::new(20.0, 6.5), Point::new(8.0, 6.5)
        )));
        assert!(LineIntersection::Exits {
            point: Point::new(10.0 - 4.0 * hsq3, 6.5),
        }.nearly_equal(oval.find_line_intersection(
            Point::new(12.0, 6.5), Point::new(0.0, 6.5)
        )));

        // Vertical lines going through the center
        assert!(LineIntersection::Crosses {
            entrance: Point::new(10.0, 2.0),
            exit: Point::new(10.0, 8.0)
        }.nearly_equal(oval.find_line_intersection(
            Point::new(10.0, -10.0), Point::new(10.0, 123.0)
        )));
        assert!(LineIntersection::Exits {
            point: Point::new(10.0, 8.0)
        }.nearly_equal(oval.find_line_intersection(
            Point::new(10.0, 3.0), Point::new(10.0, 123.0)
        )));
        assert!(LineIntersection::Enters {
            point: Point::new(10.0, 2.0),
        }.nearly_equal(oval.find_line_intersection(
            Point::new(10.0, -10.0), Point::new(10.0, 7.0)
        )));

        // The last 3 lines, but in the other direction
        assert!(LineIntersection::Crosses {
            exit: Point::new(10.0, 2.0),
            entrance: Point::new(10.0, 8.0)
        }.nearly_equal(oval.find_line_intersection(
            Point::new(10.0, 123.0), Point::new(10.0, -10.0)
        )));
        assert!(LineIntersection::Enters {
            point: Point::new(10.0, 8.0)
        }.nearly_equal(oval.find_line_intersection(
            Point::new(10.0, 123.0), Point::new(10.0, 3.0)
        )));
        assert!(LineIntersection::Exits {
            point: Point::new(10.0, 2.0),
        }.nearly_equal(oval.find_line_intersection(
            Point::new(10.0, 7.0), Point::new(10.0, -10.0)
        )));
    }
}