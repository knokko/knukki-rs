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
        //let distance = from.distance_to(to);
        let distance = 1.0; //TODO Clean up
        let direction_x = (to.get_x() - from.get_x()) / distance;
        let direction_y = (to.get_y() - from.get_y()) / distance;

        /*
         * To keep things short, use:
         * dx = direction_x, dy = direction_y, cx = center.x, cy = center.y, fx = from.x,
         * fy = from.y, rx = radius_x, ry = radius_y
         * with the unknown L('s)
         *
         * wx = weighted distance X = (fx + L*dx - cx) / rx
         * wy = weighted distance Y = (fy + L*dy - cy) / ry
         *
         * We need to solve wx * wx + wy * wy = 1.0:
         * (fx + L*dx - cx)^2 / rx^2 + (fy + L*dy - cy)^2 / ry^2 = 1.0
         * To keep things short, use hx = fx - cx and hy = fy - cy
         * (L*dx + hx)^2 / rx^2 + (L*dy + hy)^2 / ry^2 = 1.0
         * (L^2*dx^2 + 2*L*dx*hx + hx^2) / rx^2 + (L^2*dy^2 + 2*L*dy*hy + hy^2) / ry^2 = 1.0
         * L^2*(dx^2*rx^-2 + dy^2*ry^-2) + L*2*(dx*hx*rx^-2 + dy*hy*ry^-2) + hx^2*rx^-2 + hy^2*ry^-2 = 1.0
         *
         * Solve this with the quadratic formula:
         * a = (dx^2*rx^-2 + dy^2*ry^-2)
         * b = 2*(dx*hx*rx^-2 + dy*hy*ry^-2)
         * c = hx^2*rx^-2 + hy^2*ry^-2 - 1.0
         *
         * D = b^2 - 4*a*c
         * L = (-b +- sqrt(D)) / (2*a) if D > 0.0
         * I will ignore the D = 0.0 case since it's not reliable due to rounding errors. When D is
         * 0.0, I will consider it as a 'miss'
         */

        let helper_x = from.get_x() - self.center.get_x();
        let helper_y = from.get_y() - self.center.get_y();

        let a_x = direction_x * direction_x / (self.radius_x * self.radius_x);
        let a_y = (direction_y * direction_y) / (self.radius_y * self.radius_y);
        let a = a_x + a_y;

        let b_x = direction_x * helper_x / (self.radius_x * self.radius_x);
        let b_y = direction_y * helper_y / (self.radius_y * self.radius_y);
        let b = 2.0 * (b_x + b_y);

        let c_x = helper_x * helper_x / (self.radius_x * self.radius_x);
        let c_y = helper_y * helper_y / (self.radius_y * self.radius_y);
        let c = c_x + c_y - 1.0;

        let discriminant = b * b - 4.0 * a * c;
        return if discriminant > 0.0 {
            // The line would cross the circle if it had unbounded length
            let lambda1 = (-b - discriminant.sqrt()) / (2.0 * a);
            let lambda2 = (-b + discriminant.sqrt()) / (2.0 * a);

            let x1 = from.get_x() + lambda1 * direction_x;
            let y1 = from.get_y() + lambda1 * direction_y;
            let point1 = Point::new(x1, y1);

            let x2 = from.get_x() + lambda2 * direction_x;
            let y2 = from.get_y() + lambda2 * direction_y;
            let point2 = Point::new(x2, y2);

            if lambda1 <= 0.0 {
                // The line can't enter the oval
                if lambda2 < 0.0 {
                    // The line ends before it would intersect the oval
                    LineIntersection::FullyOutside
                } else {
                    // The line exits the oval or is entirely inside it
                    if lambda2 > 1.0 {
                        // The line is entirely inside the oval
                        LineIntersection::FullyInside
                    } else {
                        // The line exits the oval
                        LineIntersection::Exits {
                            point: point2
                        }
                    }
                }
            } else {
                // The line can't exit the oval
                if lambda1 <= 1.0 {
                    // The line enters or crosses the oval
                    if lambda2 <= 1.0 {
                        // The line crosses the oval
                        LineIntersection::Crosses {
                            entrance: point1,
                            exit: point2
                        }
                    } else {
                        // The line enters the oval
                        LineIntersection::Enters {
                            point: point1
                        }
                    }
                } else {
                    // The line ends before it could intersect the oval
                    LineIntersection::FullyOutside
                }
            }
        } else {
            // The line won't cross the circle
            LineIntersection::FullyOutside
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