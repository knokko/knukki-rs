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
    fn is_inside(&self, point: Point) -> bool {
        self.is_within_bounds(point)
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

    fn find_line_intersection(&self, from: Point, to: Point) -> LineIntersection {
        let dx = to.get_x() - from.get_x();
        let dy = to.get_y() - from.get_y();

        let from_inside = self.is_within_bounds(from);
        let to_inside = self.is_within_bounds(to);

        // This is the easy case
        if from_inside && to_inside {
            return LineIntersection::FullyInside;
        }

        // Use case distinction to avoid divisions by 0 (or numbers like 0.0001)
        // TODO Test both cases
        if dx.abs() > dy.abs() {
            // Express line formula as: y = slope * x + adder
            let slope = dy / dx;
            let adder = from.get_y() - slope * from.get_x();

            let left_y = slope * self.left + adder;
            let right_y = slope * self.right + adder;

            // The line goes from a point inside this rectangle to a point outside it
            if from_inside {
                // We need to find the intersection of the line with this rectangle
                return if dx > 0.0 {
                    if right_y >= self.bottom && right_y <= self.top {
                        LineIntersection::Exits { point: Point::new(self.right, right_y) }
                    } else if dy > 0.0 {
                        // Solve: slope * x + adder = self.top
                        // Solve: slope * x = self.top - adder
                        // Solution: x = (self.top - adder) / slope
                        let top_x = (self.top - adder) / slope;
                        LineIntersection::Exits { point: Point::new(top_x, self.top) }
                    } else {
                        // Solve: slope * x + adder = self.bottom
                        let bottom_x = (self.bottom - adder) / slope;
                        LineIntersection::Exits { point: Point::new(bottom_x, self.bottom) }
                    }
                } else {
                    if left_y >= self.bottom && left_y <= self.top {
                        LineIntersection::Exits { point: Point::new(self.left, left_y) }
                    } else if dy > 0.0 {
                        let top_x = (self.top - adder) / slope;
                        LineIntersection::Exits { point: Point::new(top_x, self.top) }
                    } else {
                        let bottom_x = (self.bottom - adder) / slope;
                        LineIntersection::Exits { point: Point::new(bottom_x, self.bottom) }
                    }
                }
            }

            // The line goes from a point outside this rectangle to a point inside it
            if to_inside {
                return if dx > 0.0 {
                    if left_y >= self.bottom && left_y <= self.top {
                        LineIntersection::Enters { point: Point::new(self.left, left_y) }
                    } else if dy > 0.0 {
                        // Solve: slope * x + adder = self.bottom
                        // Solve: slope * x = self.bottom - adder
                        // Solution: x = (self.bottom - adder) / slope
                        let bottom_x = (self.bottom - adder) / slope;
                        LineIntersection::Enters { point: Point::new(bottom_x, self.bottom) }
                    } else {
                        let top_x = (self.top - adder) / slope;
                        LineIntersection::Enters { point: Point::new(top_x, self.top) }
                    }
                } else {
                    if right_y >= self.bottom && right_y <= self.top {
                        LineIntersection::Enters { point: Point::new(self.right, right_y) }
                    } else if dy > 0.0 {
                        let bottom_x = (self.bottom - adder) / slope;
                        LineIntersection::Enters { point: Point::new(bottom_x, self.bottom) }
                    } else {
                        let top_x = (self.top - adder) / slope;
                        LineIntersection::Enters { point: Point::new(top_x, self.top) }
                    }
                }
            }

            // If we reach this part, the line goes from a point outside this rectangle to another
            // point outside this rectangle. We need to check if it intersects this rectangle
            // between those points.
            let min_x = f32::min(from.get_x(), to.get_x());
            let max_x = f32::max(from.get_x(), to.get_x());
            let min_y = f32::min(from.get_y(), to.get_y());
            let max_y = f32::max(from.get_y(), to.get_y());
            if max_x < self.left || max_y < self.bottom || min_x > self.right || min_y > self.top {
                return LineIntersection::FullyOutside;
            }

            return if left_y < self.bottom {
                if right_y < self.bottom {
                    // For any `self.left <= x <= self.right`, the y-coordinate on the line is
                    // below this rectangle
                    LineIntersection::FullyOutside
                } else if right_y > self.top {
                    // The line crosses the top and bottom of this rectangle
                    let bottom_x = (self.bottom - adder) / slope;
                    let top_x = (self.top - adder) / slope;
                    if to.get_y() > from.get_y() {
                        LineIntersection::Crosses {
                            entrance: Point::new(bottom_x, self.bottom),
                            exit: Point::new(top_x, self.top)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(bottom_x, self.bottom),
                            entrance: Point::new(top_x, self.top)
                        }
                    }

                } else {
                    // The line crosses the bottom and right of this rectangle
                    let bottom_x = (self.bottom - adder) / slope;
                    if to.get_y() > from.get_y() {
                        LineIntersection::Crosses {
                            entrance: Point::new(bottom_x, self.bottom),
                            exit: Point::new(self.right, right_y)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(bottom_x, self.bottom),
                            entrance: Point::new(self.right, right_y)
                        }
                    }
                }
            } else if left_y > self.top {
                if right_y > self.top {
                    // For any `self.left <= x <= self.right`, the y-coordinate on the line is
                    // above this rectangle
                    LineIntersection::FullyOutside
                } else if right_y < self.bottom {
                    // The line crosses the top and bottom of this rectangle
                    let bottom_x = (self.bottom - adder) / slope;
                    let top_x = (self.top - adder) / slope;
                    if to.get_y() > from.get_y() {
                        LineIntersection::Crosses {
                            entrance: Point::new(bottom_x, self.bottom),
                            exit: Point::new(top_x, self.top)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(bottom_x, self.bottom),
                            entrance: Point::new(top_x, self.top)
                        }
                    }

                } else {
                    // The line crosses the top and right of this rectangle
                    let top_x = (self.top - adder) / slope;
                    if to.get_x() > from.get_x() {
                        LineIntersection::Crosses {
                            entrance: Point::new(top_x, self.top),
                            exit: Point::new(self.right, right_y)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(top_x, self.top),
                            entrance: Point::new(self.right, right_y)
                        }
                    }
                }
            } else {
                if right_y > self.top {
                    // The line crosses the left and the top of this rectangle
                    let top_x = (self.top - adder) / slope;
                    if to.get_y() > from.get_y() {
                        LineIntersection::Crosses {
                            entrance: Point::new(self.left, left_y),
                            exit: Point::new(top_x, self.top)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(self.left, left_y),
                            entrance: Point::new(top_x, self.top)
                        }
                    }

                } else if right_y < self.bottom {
                    // The line crosses the left and bottom of this rectangle
                    let bottom_x = (self.bottom - adder) / slope;
                    if to.get_x() > from.get_x() {
                        LineIntersection::Crosses {
                            entrance: Point::new(self.left, left_y),
                            exit: Point::new(bottom_x, self.bottom)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(self.left, left_y),
                            entrance: Point::new(bottom_x, self.bottom)
                        }
                    }
                } else {
                    // The line crosses the left and right of this rectangle
                    if to.get_x() > from.get_x() {
                        LineIntersection::Crosses {
                            entrance: Point::new(self.left, left_y),
                            exit: Point::new(self.right, right_y)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(self.left, left_y),
                            entrance: Point::new(self.right, right_y)
                        }
                    }
                }
            }
        } else {
            // Express line formula as: x = slopeInverse * y + adder
            let slopeInverse = dx / dy;
            let adder = from.get_x() - slopeInverse * from.get_y();

            let bottom_x = slopeInverse * self.bottom + adder;
            let top_x = slopeInverse * self.top + adder;

            // The line goes from a point inside this rectangle to a point outside it
            if from_inside {
                // We need to find the intersection of the line with this rectangle
                return if dy > 0.0 {
                    if top_x >= self.left && top_x <= self.right {
                        LineIntersection::Exits { point: Point::new(top_x, self.top) }
                    } else if dx > 0.0 {
                        // Solve: slopeInverse * y + adder = self.right
                        // Solve: slopeInverse * y = self.right - adder
                        // Solution: y = (self.right - adder) / slopeInverse
                        let right_y = (self.right - adder) / slopeInverse;
                        LineIntersection::Exits { point: Point::new(self.right, right_y) }
                    } else {
                        let left_y = (self.left - adder) / slopeInverse;
                        LineIntersection::Exits { point: Point::new(self.left, left_y) }
                    }
                } else {
                    if bottom_x >= self.left && bottom_x <= self.right {
                        LineIntersection::Exits { point: Point::new(bottom_x, self.bottom) }
                    } else if dx > 0.0 {
                        let right_y = (self.right - adder) / slopeInverse;
                        LineIntersection::Exits { point: Point::new(self.right, right_y) }
                    } else {
                        let left_y = (self.left - adder) / slopeInverse;
                        LineIntersection::Exits { point: Point::new(self.left, left_y) }
                    }
                }
            }

            // The line goes from a point outside this rectangle to a point inside it
            if to_inside {
                return if dy < 0.0 {
                    if top_x >= self.left && top_x <= self.right {
                        LineIntersection::Enters { point: Point::new(top_x, self.top) }
                    } else if dx > 0.0 {
                        // Solve: slopeInverse * y + adder = self.right
                        // Solve: slopeInverse * y = self.right - adder
                        // Solution: y = (self.right - adder) / slopeInverse
                        let right_y = (self.right - adder) / slopeInverse;
                        LineIntersection::Enters { point: Point::new(self.right, right_y) }
                    } else {
                        let left_y = (self.left - adder) / slopeInverse;
                        LineIntersection::Enters { point: Point::new(self.left, left_y) }
                    }
                } else {
                    if bottom_x >= self.left && bottom_x <= self.right {
                        LineIntersection::Enters { point: Point::new(bottom_x, self.bottom) }
                    } else if dx > 0.0 {
                        let right_y = (self.right - adder) / slopeInverse;
                        LineIntersection::Enters { point: Point::new(self.right, right_y) }
                    } else {
                        let left_y = (self.left - adder) / slopeInverse;
                        LineIntersection::Enters { point: Point::new(self.left, left_y) }
                    }
                }
            }

            // If we reach this part, the line goes from a point outside this rectangle to another
            // point outside this rectangle. We need to check if it intersects this rectangle
            // between those points.
            let min_x = f32::min(from.get_x(), to.get_x());
            let max_x = f32::max(from.get_x(), to.get_x());
            let min_y = f32::min(from.get_y(), to.get_y());
            let max_y = f32::max(from.get_y(), to.get_y());
            if max_x < self.left || max_y < self.bottom || min_x > self.right || min_y > self.top {
                return LineIntersection::FullyOutside;
            }

            return if bottom_x < self.left {
                if top_x < self.left {
                    // For any `self.bottom <= y <= self.top`, the x-coordinate on the line is
                    // on the left of this rectangle
                    LineIntersection::FullyOutside
                } else if top_x > self.right {
                    // The line crosses the left and right of this rectangle
                    let left_y = (self.left - adder) / slopeInverse;
                    let right_y = (self.right - adder) / slopeInverse;
                    if to.get_x() > from.get_x() {
                        LineIntersection::Crosses {
                            entrance: Point::new(self.left, left_y),
                            exit: Point::new(self.right, right_y)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(self.left, left_y),
                            entrance: Point::new(self.right, right_y)
                        }
                    }

                } else {
                    // The line crosses the left and top of this rectangle
                    let left_y = (self.left - adder) / slopeInverse;
                    if to.get_x() > from.get_x() {
                        LineIntersection::Crosses {
                            entrance: Point::new(self.left, left_y),
                            exit: Point::new(top_x, self.top)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(self.left, left_y),
                            entrance: Point::new(top_x, self.top)
                        }
                    }
                }
            } else if bottom_x > self.right {
                if top_x > self.right {
                    // For any `self.bottom <= y <= self.top`, the x-coordinate on the line is
                    // on the right of this rectangle
                    LineIntersection::FullyOutside
                } else if top_x < self.left {
                    // The line crosses the left and right of this rectangle
                    let left_y = (self.left - adder) / slopeInverse;
                    let right_y = (self.right - adder) / slopeInverse;
                    if to.get_x() > from.get_x() {
                        LineIntersection::Crosses {
                            entrance: Point::new(self.left, left_y),
                            exit: Point::new(self.right, right_y)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(self.left, left_y),
                            entrance: Point::new(self.right, right_y)
                        }
                    }
                } else {
                    // The line crosses the top and right of this rectangle
                    let right_y = (self.right - adder) / slopeInverse;
                    if to.get_x() > from.get_x() {
                        LineIntersection::Crosses {
                            entrance: Point::new(top_x, self.top),
                            exit: Point::new(self.right, right_y)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(top_x, self.top),
                            entrance: Point::new(self.right, right_y)
                        }
                    }
                }
            } else {
                // self.left <= bottom_x <= self.right
                if top_x > self.right {
                    // The line crosses the bottom and the right of this rectangle
                    let right_y = (self.right - adder) / slopeInverse;
                    if to.get_y() > from.get_y() {
                        LineIntersection::Crosses {
                            entrance: Point::new(bottom_x, self.bottom),
                            exit: Point::new(self.right, right_y)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(bottom_x, self.bottom),
                            entrance: Point::new(self.right, right_y)
                        }
                    }

                } else if top_x < self.left {
                    // The line crosses the left and bottom of this rectangle
                    let left_y = (self.left - adder) / slopeInverse;
                    if to.get_x() > from.get_x() {
                        LineIntersection::Crosses {
                            entrance: Point::new(self.left, left_y),
                            exit: Point::new(bottom_x, self.bottom)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(self.left, left_y),
                            entrance: Point::new(bottom_x, self.bottom)
                        }
                    }
                } else {
                    // The line crosses the bottom and top of this rectangle
                    if to.get_y() > from.get_y() {
                        LineIntersection::Crosses {
                            entrance: Point::new(bottom_x, self.bottom),
                            exit: Point::new(top_x, self.top)
                        }
                    } else {
                        LineIntersection::Crosses {
                            exit: Point::new(bottom_x, self.bottom),
                            entrance: Point::new(top_x, self.top)
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_is_inside() {
        let rect = RectangularDrawnRegion::new(-0.2, -0.4, 0.6, 1.0);

        // Boundary cases
        assert!(rect.is_inside(Point::new(-0.2, -0.4)));
        assert!(rect.is_inside(Point::new(0.6, 1.0)));
        assert!(!rect.is_inside(Point::new(-0.21, 0.0)));
        assert!(!rect.is_inside(Point::new(0.0, 1.01)));

        // Simpler cases
        assert!(rect.is_inside(Point::new(0.0, 0.0)));
        assert!(!rect.is_inside(Point::new(2.0, -3.5)));
    }

    #[test]
    fn test_invalid() {
        let rect = RectangularDrawnRegion::new(1.0, 1.0, -1.0, -1.0);

        assert!(!rect.is_inside(Point::new(0.0, 0.0)));
        assert!(!rect.is_inside(Point::new(1.0, 1.0)));
    }

    #[test]
    fn test_bounds() {
        let rect = RectangularDrawnRegion::new(0.3, 0.8, 1.0, 1.5);

        assert_eq!(0.3, rect.get_left());
        assert_eq!(0.8, rect.get_bottom());
        assert_eq!(1.0, rect.get_right());
        assert_eq!(1.5, rect.get_top());
    }

    fn li_exit(exit_x: f32, exit_y: f32) -> LineIntersection {
        LineIntersection::Exits { point: Point::new(exit_x, exit_y)}
    }

    fn fli(rect: &RectangularDrawnRegion, from: Point, to: Point) -> LineIntersection {
        rect.find_line_intersection(from, to)
    }

    #[test]
    fn test_line_intersection_from_inside_to_outside() {
        let rect = RectangularDrawnRegion::new(
            30.0, 10.0, 100.0, 20.0
        );

        let middle = Point::new(65.0, 15.0);
        let near_top = Point::new(65.0, 19.0);
        let near_bottom = Point::new(65.0, 11.0);

        // Test lines from the middle to the right
        assert!(li_exit(100.0, 16.0).nearly_equal(fli(&rect, middle, Point::new(135.0, 17.0))));
        assert!(li_exit(100.0, 15.0).nearly_equal(fli(&rect, middle, Point::new(135.0, 15.0))));
        assert!(li_exit(100.0, 14.0).nearly_equal(fli(&rect, middle, Point::new(135.0, 13.0))));
        assert!(li_exit(66.0, 20.0).nearly_equal(fli(&rect, middle, Point::new(67.0, 25.0))));

        // Test vertical lines from the middle
        assert!(li_exit(65.0, 20.0).nearly_equal(fli(&rect, middle, Point::new(65.0, 25.0))));
        assert!(li_exit(65.0, 10.0).nearly_equal(fli(&rect, middle, Point::new(65.0, 5.0))));

        // Test lines from the middle to the left
        assert!(li_exit(30.0, 16.0).nearly_equal(fli(&rect, middle, Point::new(-5.0, 17.0))));
        assert!(li_exit(30.0, 15.0).nearly_equal(fli(&rect, middle, Point::new(-5.0, 15.0))));
        assert!(li_exit(30.0, 14.0).nearly_equal(fli(&rect, middle, Point::new(-5.0, 13.0))));
        assert!(li_exit(64.0, 20.0).nearly_equal(fli(&rect, middle, Point::new(63.0, 25.0))));

        // Test lines from the top to the right
        assert!(li_exit(90.0, 20.0).nearly_equal(fli(&rect, near_top, Point::new(115.0, 21.0))));
        assert!(li_exit(100.0, 19.0).nearly_equal(fli(&rect, near_top, Point::new(115.0, 19.0))));
        assert!(li_exit(100.0, 18.0).nearly_equal(fli(&rect, near_top, Point::new(135.0, 17.0))));

        // Test lines from the top to the left
        assert!(li_exit(40.0, 20.0).nearly_equal(fli(&rect, near_top, Point::new(15.0, 21.0))));
        assert!(li_exit(30.0, 19.0).nearly_equal(fli(&rect, near_top, Point::new(-5.0, 19.0))));
        assert!(li_exit(30.0, 18.0).nearly_equal(fli(&rect, near_top, Point::new(-5.0, 17.0))));

        // Test lines from the bottom to the right
        assert!(li_exit(100.0, 12.0).nearly_equal(fli(&rect, near_bottom, Point::new(135.0, 13.0))));
        assert!(li_exit(100.0, 11.0).nearly_equal(fli(&rect, near_bottom, Point::new(115.0, 11.0))));
        assert!(li_exit(90.0, 10.0).nearly_equal(fli(&rect, near_bottom, Point::new(115.0, 9.0))));

        // Test lines from the bottom to the left
        assert!(li_exit(30.0, 12.0).nearly_equal(fli(&rect, near_bottom, Point::new(-5.0, 13.0))));
        assert!(li_exit(30.0, 11.0).nearly_equal(fli(&rect, near_bottom, Point::new(-5.0, 11.0))));
        assert!(li_exit(40.0, 10.0).nearly_equal(fli(&rect, near_bottom, Point::new(15.0, 9.0))));
    }

    fn li_enter(enter_x: f32, enter_y: f32) -> LineIntersection {
        LineIntersection::Enters { point: Point::new(enter_x, enter_y)}
    }

    #[test]
    fn test_line_intersection_from_outside_to_inside() {
        let rect = RectangularDrawnRegion::new(
            30.0, 10.0, 100.0, 20.0
        );

        let middle = Point::new(65.0, 15.0);
        let near_top = Point::new(65.0, 19.0);
        let near_bottom = Point::new(65.0, 11.0);

        // Test lines from the middle to the right
        assert!(li_enter(100.0, 16.0).nearly_equal(fli(&rect, Point::new(135.0, 17.0), middle)));
        assert!(li_enter(100.0, 15.0).nearly_equal(fli(&rect, Point::new(135.0, 15.0), middle)));
        assert!(li_enter(100.0, 14.0).nearly_equal(fli(&rect, Point::new(135.0, 13.0), middle)));
        assert!(li_enter(66.0, 20.0).nearly_equal(fli(&rect, Point::new(67.0, 25.0), middle)));

        // Test vertical lines from the middle
        assert!(li_enter(65.0, 20.0).nearly_equal(fli(&rect, Point::new(65.0, 25.0), middle)));
        assert!(li_enter(65.0, 10.0).nearly_equal(fli(&rect, Point::new(65.0, 5.0), middle)));

        // Test lines from the middle to the left
        assert!(li_enter(30.0, 16.0).nearly_equal(fli(&rect, Point::new(-5.0, 17.0), middle)));
        assert!(li_enter(30.0, 15.0).nearly_equal(fli(&rect, Point::new(-5.0, 15.0), middle)));
        assert!(li_enter(30.0, 14.0).nearly_equal(fli(&rect, Point::new(-5.0, 13.0), middle)));
        assert!(li_enter(64.0, 20.0).nearly_equal(fli(&rect, Point::new(63.0, 25.0), middle)));

        // Test lines from the top to the right
        assert!(li_enter(90.0, 20.0).nearly_equal(fli(&rect, Point::new(115.0, 21.0), near_top)));
        assert!(li_enter(100.0, 19.0).nearly_equal(fli(&rect, Point::new(115.0, 19.0), near_top)));
        assert!(li_enter(100.0, 18.0).nearly_equal(fli(&rect, Point::new(135.0, 17.0), near_top)));

        // Test lines from the top to the left
        assert!(li_enter(40.0, 20.0).nearly_equal(fli(&rect, Point::new(15.0, 21.0), near_top)));
        assert!(li_enter(30.0, 19.0).nearly_equal(fli(&rect, Point::new(-5.0, 19.0), near_top)));
        assert!(li_enter(30.0, 18.0).nearly_equal(fli(&rect, Point::new(-5.0, 17.0), near_top)));

        // Test lines from the bottom to the right
        assert!(li_enter(100.0, 12.0).nearly_equal(fli(&rect, Point::new(135.0, 13.0), near_bottom)));
        assert!(li_enter(100.0, 11.0).nearly_equal(fli(&rect, Point::new(115.0, 11.0), near_bottom)));
        assert!(li_enter(90.0, 10.0).nearly_equal(fli(&rect, Point::new(115.0, 9.0), near_bottom)));

        // Test lines from the bottom to the left
        assert!(li_enter(30.0, 12.0).nearly_equal(fli(&rect, Point::new(-5.0, 13.0), near_bottom)));
        assert!(li_enter(30.0, 11.0).nearly_equal(fli(&rect, Point::new(-5.0, 11.0), near_bottom)));
        assert!(li_enter(40.0, 10.0).nearly_equal(fli(&rect, Point::new(15.0, 9.0), near_bottom)));
    }

    #[test]
    fn test_line_intersection_fully_inside() {
        let rect = RectangularDrawnRegion::new(1.0, 5.0, 5.0, 7.0);
        let lii = LineIntersection::FullyInside;

        // Horizontal lines
        assert_eq!(lii, rect.find_line_intersection(Point::new(3.0, 6.0), Point::new(4.0, 6.0)));
        assert_eq!(lii, rect.find_line_intersection(Point::new(3.0, 6.0), Point::new(2.0, 6.0)));

        // Vertical lines
        assert_eq!(lii, rect.find_line_intersection(Point::new(3.0, 6.0), Point::new(3.0, 6.5)));
        assert_eq!(lii, rect.find_line_intersection(Point::new(3.0, 6.0), Point::new(3.0, 5.5)));

        // Lines to the right
        assert_eq!(lii, rect.find_line_intersection(Point::new(3.0, 6.0), Point::new(4.0, 6.5)));
        assert_eq!(lii, rect.find_line_intersection(Point::new(3.0, 6.0), Point::new(3.1, 6.5)));
        assert_eq!(lii, rect.find_line_intersection(Point::new(3.0, 6.0), Point::new(4.0, 5.5)));
        assert_eq!(lii, rect.find_line_intersection(Point::new(3.0, 6.0), Point::new(3.1, 5.5)));

        // Lines to the left
        assert_eq!(lii, rect.find_line_intersection(Point::new(3.0, 6.0), Point::new(2.0, 6.5)));
        assert_eq!(lii, rect.find_line_intersection(Point::new(3.0, 6.0), Point::new(2.0, 5.5)));
        assert_eq!(lii, rect.find_line_intersection(Point::new(3.0, 6.0), Point::new(2.9, 6.5)));
        assert_eq!(lii, rect.find_line_intersection(Point::new(3.0, 6.0), Point::new(2.9, 5.5)));
    }

    #[test]
    fn test_line_intersection_fully_outside() {
        let lio = LineIntersection::FullyOutside;
        let rect = RectangularDrawnRegion::new(0.0, 3.0, 5.0, 10.0);

        // Horizontal lines to the right
        assert_eq!(lio, rect.find_line_intersection(Point::new(-5.0, 2.0), Point::new(10.0, 2.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(-5.0, 11.0), Point::new(10.0, 11.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(-5.0, 5.0), Point::new(-1.0, 5.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(6.0, 5.0), Point::new(8.0, 5.0)));

        // Horizontal lines to the left
        assert_eq!(lio, rect.find_line_intersection(Point::new(10.0, 2.0), Point::new(-5.0, 2.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(10.0, 11.0), Point::new(-5.0, 11.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(-1.0, 5.0), Point::new(-5.0, 5.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(8.0, 5.0), Point::new(6.0, 5.0)));

        // Upwards vertical lines
        assert_eq!(lio, rect.find_line_intersection(Point::new(2.0, -10.0), Point::new(2.0, 2.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(-1.0, 2.0), Point::new(-1.0, 8.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(6.0, 4.0), Point::new(6.0, 6.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(2.0, 11.0), Point::new(2.0, 20.0)));

        // Downward vertical lines
        assert_eq!(lio, rect.find_line_intersection(Point::new(2.0, 2.0), Point::new(2.0, -10.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(-1.0, 8.0), Point::new(-1.0, 2.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(6.0, 6.0), Point::new(6.0, 4.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(2.0, 20.0), Point::new(2.0, 11.0)));

        // Right-up lines
        assert_eq!(lio, rect.find_line_intersection(Point::new(-1.0, 2.0), Point::new(5.0, 2.5)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(-1.0, -12.0), Point::new(5.0, 2.5)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(-5.0, 7.0), Point::new(-0.1, 10.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(-5.0, 1.0), Point::new(-0.1, 10.0)));

        // Right-down lines
        assert_eq!(lio, rect.find_line_intersection(Point::new(-5.0, 5.0), Point::new(0.0, 2.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(-5.0, 9.0), Point::new(0.0, 2.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(2.0, 12.0), Point::new(5.5, 10.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(2.0, 17.0), Point::new(5.5, 10.0)));

        // Left-down lines
        assert_eq!(lio, rect.find_line_intersection(Point::new(5.0, 2.5), Point::new(-1.0, 2.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(5.0, 2.5), Point::new(-1.0, -12.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(-0.1, 10.0), Point::new(-5.0, 7.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(-0.1, 10.0), Point::new(-5.0, 1.0)));

        // Left-up lines
        assert_eq!(lio, rect.find_line_intersection(Point::new(0.0, 2.0), Point::new(-5.0, 5.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(0.0, 2.0), Point::new(-5.0, 9.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(5.5, 10.0), Point::new(2.0, 12.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(5.5, 10.0), Point::new(2.0, 17.0)));
    }
    
    fn li_cross(enter_x: f32, enter_y: f32, exit_x: f32, exit_y: f32) -> LineIntersection {
        LineIntersection::Crosses {
            entrance: Point::new(enter_x, enter_y),
            exit: Point::new(exit_x, exit_y)
        }
    }

    #[test]
    fn test_line_intersection_crossing() {
        let rect = RectangularDrawnRegion::new(
            30.0, 10.0, 100.0, 20.0
        );

        let middle = Point::new(65.0, 15.0);
        let near_top = Point::new(65.0, 19.0);
        let near_bottom = Point::new(65.0, 11.0);

        // Test lines through the middle to the right
        assert!(li_cross(30.0, 14.0, 100.0, 16.0)
            .nearly_equal(fli(&rect, Point::new(-5.0, 13.0), Point::new(135.0, 17.0))));
        assert!(li_cross(30.0, 15.0, 100.0, 15.0)
            .nearly_equal(fli(&rect, Point::new(-5.0, 15.0), Point::new(135.0, 15.0))));
        assert!(li_cross(30.0, 16.0, 100.0, 14.0)
            .nearly_equal(fli(&rect, Point::new(-5.0, 17.0), Point::new(135.0, 13.0))));
        assert!(li_cross(64.0, 10.0, 66.0, 20.0)
            .nearly_equal(fli(&rect, Point::new(63.0, 5.0), Point::new(67.0, 25.0))));

        // Test vertical lines through the middle
        assert!(li_cross(65.0, 10.0, 65.0, 20.0)
            .nearly_equal(fli(&rect, Point::new(65.0, 5.0), Point::new(65.0, 25.0))));
        assert!(li_cross(65.0, 20.0, 65.0, 10.0)
            .nearly_equal(fli(&rect, Point::new(65.0, 25.0), Point::new(65.0, 5.0))));

        // Test lines through the middle to the left
        assert!(li_cross(100.0, 14.0, 30.0, 16.0)
            .nearly_equal(fli(&rect, Point::new(135.0, 13.0), Point::new(-5.0, 17.0))));
        assert!(li_cross(100.0, 15.0, 30.0, 15.0)
            .nearly_equal(fli(&rect, Point::new(135.0, 15.0), Point::new(-5.0, 15.0))));
        assert!(li_cross(100.0, 16.0, 30.0, 14.0)
            .nearly_equal(fli(&rect, Point::new(135.0, 17.0), Point::new(-5.0, 13.0))));
        assert!(li_cross(66.0, 10.0, 64.0, 20.0)
            .nearly_equal(fli(&rect, Point::new(67.0, 5.0), Point::new(63.0, 25.0))));

        // Test lines through the top to the right
        assert!(li_cross(30.0, 17.6, 90.0, 20.0)
            .nearly_equal(fli(&rect, Point::new(15.0, 17.0), Point::new(115.0, 21.0))));
        assert!(li_cross(30.0, 19.0, 100.0, 19.0)
            .nearly_equal(fli(&rect, Point::new(0.0, 19.0), Point::new(115.0, 19.0))));
        assert!(li_cross(30.0, 20.0, 100.0, 18.0)
            .nearly_equal(fli(&rect, Point::new(-5.0, 21.0), Point::new(135.0, 17.0))));

        // Test lines through the top to the left
        assert!(li_cross(100.0, 17.6, 40.0, 20.0)
            .nearly_equal(fli(&rect, Point::new(115.0, 17.0), Point::new(15.0, 21.0))));
        assert!(li_cross(100.0, 19.0, 30.0, 19.0)
            .nearly_equal(fli(&rect, Point::new(135.0, 19.0), Point::new(-5.0, 19.0))));
        assert!(li_cross(100.0, 20.0, 30.0, 18.0)
            .nearly_equal(fli(&rect, Point::new(135.0, 21.0), Point::new(-5.0, 17.0))));

        // Test lines through the bottom to the right
        assert!(li_cross(30.0, 10.0, 100.0, 12.0)
            .nearly_equal(fli(&rect, Point::new(-5.0, 9.0), Point::new(135.0, 13.0))));
        assert!(li_cross(30.0, 11.0, 100.0, 11.0)
            .nearly_equal(fli(&rect, Point::new(-5.0, 11.0), Point::new(115.0, 11.0))));
        assert!(li_cross(30.0, 12.4, 90.0, 10.0)
            .nearly_equal(fli(&rect, Point::new(15.0, 13.0), Point::new(115.0, 9.0))));

        // Test lines through the bottom to the left
        assert!(li_cross(100.0, 10.0, 30.0, 12.0)
            .nearly_equal(fli(&rect, Point::new(135.0, 9.0), Point::new(-5.0, 13.0))));
        assert!(li_cross(100.0, 11.0, 30.0, 11.0)
            .nearly_equal(fli(&rect, Point::new(135.0, 11.0), Point::new(-5.0, 11.0))));
        assert!(li_cross(100.0, 12.4, 40.0, 10.0)
            .nearly_equal(fli(&rect, Point::new(115.0, 13.0), Point::new(15.0, 9.0))));
    }

    #[test]
    fn test_line_intersection_edge_cases() {
        let lio = LineIntersection::FullyOutside;
        let lii = LineIntersection::FullyInside;
        let rect = RectangularDrawnRegion::new(0.0, 3.0, 5.0, 10.0);

        // Top side, left part
        assert_eq!(lio, rect.find_line_intersection(Point::new(-5.0, 10.0), Point::new(-0.1, 10.0)));
        assert_eq!(lio, rect.find_line_intersection(Point::new(-0.1, 10.0), Point::new(-5.0, 10.0)));
        assert!(li_enter(0.0, 10.0).nearly_equal(rect.find_line_intersection(
            Point::new(-5.0, 10.0), Point::new(0.0, 10.0))));
        assert!(li_exit(0.0, 10.0).nearly_equal(rect.find_line_intersection(
            Point::new(0.0, 10.0), Point::new(-5.0, 10.0))));
        assert!(li_enter(0.0, 10.0).nearly_equal(rect.find_line_intersection(
            Point::new(-5.0, 10.0), Point::new(2.0, 10.0))));
        assert!(li_exit(0.0, 10.0).nearly_equal(rect.find_line_intersection(
            Point::new(2.0, 10.0), Point::new(-5.0, 10.0))));

        // Top side, middle part
        assert_eq!(lii, rect.find_line_intersection(Point::new(2.0, 10.0), Point::new(4.0, 10.0)));
        assert_eq!(lii, rect.find_line_intersection(Point::new(0.0, 10.0), Point::new(4.0, 10.0)));
        assert_eq!(lii, rect.find_line_intersection(Point::new(2.0, 10.0), Point::new(5.0, 10.0)));
        assert_eq!(lii, rect.find_line_intersection(Point::new(0.0, 10.0), Point::new(5.0, 10.0)));
        assert!(li_cross(0.0, 10.0, 5.0, 10.0)
            .nearly_equal(rect.find_line_intersection(Point::new(-1.0, 10.0), Point::new(6.0, 10.0))));

        // TODO Top side, right part, and the other sides
    }
    // TODO Unit tests for horizontal lines overlapping top/bottom side and vertical lines overlapping left/right side
}
