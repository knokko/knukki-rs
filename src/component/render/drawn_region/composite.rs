use crate::{DrawnRegion, LineIntersection, Point};

/// A `DrawnRegion` that is composed of other `DrawnRegion`s (typically more than
/// 1). Points will be considered *inside* a `CompositeDrawnRegion` if it is
/// *inside* at least 1 of the `DrawnRegion`s it is composed of.
pub struct CompositeDrawnRegion {
    components: Vec<Box<dyn DrawnRegion>>,

    left_bound: f32,
    bottom_bound: f32,
    right_bound: f32,
    top_bound: f32,
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
            top_bound,
        }
    }
}

impl DrawnRegion for CompositeDrawnRegion {
    fn is_inside(&self, point: Point) -> bool {
        for component in &self.components {
            if component.is_within_bounds(point) && component.is_inside(point) {
                return true;
            }
        }

        false
    }

    fn clone(&self) -> Box<dyn DrawnRegion> {
        let components = self
            .components
            .iter()
            .map(|component| component.as_ref().clone())
            .collect();
        Box::new(Self {
            components,
            left_bound: self.left_bound,
            bottom_bound: self.bottom_bound,
            right_bound: self.right_bound,
            top_bound: self.top_bound,
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

    fn find_line_intersection(&self, from: Point, to: Point) -> LineIntersection {
        let from_inside = self.is_within_bounds(from) && self.is_inside(from);
        let to_inside = self.is_within_bounds(to) && self.is_inside(to);
        return match (from_inside, to_inside) {
            (true, true) => LineIntersection::FullyInside,
            (true, false) => {
                match self.find_last_exit_point(from, to) {
                    Some(point) => LineIntersection::Exits { point },
                    // The case below could occur due to rounding errors, but should be rare
                    None => LineIntersection::FullyInside,
                }
            }
            (false, true) => {
                match self.find_first_entry_point(from, to) {
                    Some(point) => LineIntersection::Enters { point },
                    // The case below could occur due to rounding errors, but should be rare
                    None => LineIntersection::FullyOutside,
                }
            }
            (false, false) => {
                if let Some(entrance) = self.find_first_entry_point(from, to) {
                    match self.find_last_exit_point(from, to) {
                        Some(exit) => LineIntersection::Crosses { entrance, exit },
                        // The case below could occur due to rounding errors, but should be rare
                        None => LineIntersection::Enters { point: entrance },
                    }
                } else {
                    LineIntersection::FullyOutside
                }
            }
        };
    }
}

impl CompositeDrawnRegion {
    fn find_first_entry_point(&self, from: Point, to: Point) -> Option<Point> {
        let mut last_point = None;
        let mut last_distance = f32::MAX;

        for component in &self.components {
            match component.find_line_intersection(from, to) {
                LineIntersection::Enters { point } => {
                    let distance = from.distance_to(point);
                    if distance < last_distance {
                        last_point = Some(point);
                        last_distance = distance;
                    }
                }
                LineIntersection::Crosses {
                    entrance,
                    exit: _exit,
                } => {
                    let distance = from.distance_to(entrance);
                    if distance < last_distance {
                        last_point = Some(entrance);
                        last_distance = distance;
                    }
                }
                _ => {}
            };
        }

        last_point
    }

    fn find_last_exit_point(&self, from: Point, to: Point) -> Option<Point> {
        let mut last_point = None;
        let mut last_distance = f32::MAX;
        for component in &self.components {
            match component.find_line_intersection(from, to) {
                LineIntersection::Exits { point } => {
                    let distance = to.distance_to(point);
                    if distance < last_distance {
                        last_point = Some(point);
                        last_distance = distance;
                    }
                }
                LineIntersection::Crosses {
                    entrance: _entrance,
                    exit,
                } => {
                    let distance = to.distance_to(exit);
                    if distance < last_distance {
                        last_point = Some(exit);
                        last_distance = distance;
                    }
                }
                _ => {}
            };
        }

        last_point
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn test_empty() {
        let empty = CompositeDrawnRegion::new(Vec::new());
        // is_inside and is_within_bounds should always return false
        assert!(!empty.is_inside(Point::new(0.4, 14.0)));
        assert!(!empty.is_within_bounds(Point::new(0.0, 0.0)));
        assert!(!empty.is_inside(Point::new(-1.0, -2.0)));
        assert!(!empty.is_inside(Point::new(-2.0, 3.0)));
    }

    #[test]
    fn test_single() {
        let single = CompositeDrawnRegion::new(vec![Box::new(RectangularDrawnRegion::new(
            0.2, 1.0, 0.5, 2.0,
        ))]);

        assert!(!single.is_inside(Point::new(0.1, 0.9)));
        assert!(single.is_inside(Point::new(0.2, 2.0)));

        assert_eq!(0.2, single.get_left());
        assert_eq!(1.0, single.get_bottom());
        assert_eq!(0.5, single.get_right());
        assert_eq!(2.0, single.get_top());
    }

    #[test]
    fn test_double() {
        let double = CompositeDrawnRegion::new(vec![
            Box::new(RectangularDrawnRegion::new(0.0, 0.0, 1.0, 1.0)),
            Box::new(RectangularDrawnRegion::new(2.0, 1.0, 3.0, 2.0)),
        ]);

        assert!(double.is_inside(Point::new(0.1, 0.1)));
        assert!(!double.is_inside(Point::new(1.1, 0.1)));
        assert!(!double.is_inside(Point::new(2.1, 0.1)));
        assert!(!double.is_inside(Point::new(0.1, 1.1)));
        assert!(double.is_inside(Point::new(2.1, 1.1)));

        assert!(!double.is_inside(Point::new(-0.1, 0.1)));
        assert!(!double.is_inside(Point::new(3.1, 1.1)));

        assert_eq!(0.0, double.get_left());
        assert_eq!(0.0, double.get_bottom());
        assert_eq!(3.0, double.get_right());
        assert_eq!(2.0, double.get_top());
    }

    #[test]
    fn test_line_intersection_empty() {
        let region = CompositeDrawnRegion::new(Vec::new());

        // Lines should always be outside if there is not a single part
        assert_eq!(
            LineIntersection::FullyOutside,
            region.find_line_intersection(Point::new(-2.0, 5.0), Point::new(6.0, -1.0))
        );
    }

    fn test_line_intersection_code_reuse(region: &CompositeDrawnRegion) {
        assert_eq!(
            LineIntersection::FullyOutside,
            region.find_line_intersection(Point::new(6.0, 10.0), Point::new(8.0, 11.0))
        );

        assert_eq!(
            LineIntersection::FullyInside,
            region.find_line_intersection(Point::new(1.0, 4.0), Point::new(3.0, 4.0))
        );

        assert_eq!(
            LineIntersection::Enters {
                point: Point::new(0.0, 5.0)
            },
            region.find_line_intersection(Point::new(-2.0, 5.0), Point::new(2.0, 5.0))
        );

        assert_eq!(
            LineIntersection::Exits {
                point: Point::new(0.0, 5.0)
            },
            region.find_line_intersection(Point::new(2.0, 5.0), Point::new(-2.0, 5.0))
        );

        assert_eq!(
            LineIntersection::Crosses {
                entrance: Point::new(2.0, 3.0),
                exit: Point::new(2.0, 10.0)
            },
            region.find_line_intersection(Point::new(2.0, 0.0), Point::new(2.0, 20.0))
        );
    }

    #[test]
    fn test_line_intersection_single() {
        let region = CompositeDrawnRegion::new(vec![Box::new(RectangularDrawnRegion::new(
            0.0, 3.0, 5.0, 10.0,
        ))]);
        test_line_intersection_code_reuse(&region);
    }

    #[test]
    fn test_line_intersection_double() {
        let region = CompositeDrawnRegion::new(vec![
            Box::new(RectangularDrawnRegion::new(0.0, 3.0, 5.0, 10.0)),
            Box::new(RectangularDrawnRegion::new(50.0, 3.0, 55.0, 10.0)),
        ]);
        test_line_intersection_code_reuse(&region);

        assert_eq!(
            LineIntersection::FullyInside,
            region.find_line_intersection(Point::new(1.0, 5.0), Point::new(53.0, 5.0))
        );
        assert_eq!(
            LineIntersection::Enters {
                point: Point::new(0.0, 7.0)
            },
            region.find_line_intersection(Point::new(-5.0, 7.0), Point::new(52.0, 7.0))
        );
        assert_eq!(
            LineIntersection::Exits {
                point: Point::new(55.0, 7.0)
            },
            region.find_line_intersection(Point::new(4.0, 7.0), Point::new(60.0, 7.0))
        );
        assert_eq!(
            LineIntersection::Crosses {
                entrance: Point::new(0.0, 8.0),
                exit: Point::new(55.0, 8.0)
            },
            region.find_line_intersection(Point::new(-10.0, 8.0), Point::new(70.0, 8.0))
        );
    }
}
