use crate::*;

pub struct TransformedDrawnRegion<
    T: Clone + Fn(Point) -> Point + 'static,
    B: Clone + Fn(Point) -> Point + 'static,
> {
    region: Box<dyn DrawnRegion>,
    transform_function: T,
    transform_back_function: B,

    left_bound: f32,
    bottom_bound: f32,
    right_bound: f32,
    top_bound: f32,
}

impl<T: Clone + Fn(Point) -> Point + 'static, B: Clone + Fn(Point) -> Point + 'static>
    TransformedDrawnRegion<T, B>
{
    pub fn new(
        region: Box<dyn DrawnRegion>,
        transform_function: T,
        transform_back_function: B,
    ) -> Self {
        // Sanity check
        let test_point = Point::new(81.37, -35.71);
        assert!(test_point.nearly_equal(transform_back_function(transform_function(test_point))));

        // Use the transform back function to compute the transformed bounds
        let bottom_left =
            transform_back_function(Point::new(region.get_left(), region.get_bottom()));
        let top_right = transform_back_function(Point::new(region.get_right(), region.get_top()));
        Self {
            region,
            transform_function,
            transform_back_function,
            left_bound: f32::min(bottom_left.get_x(), top_right.get_x()),
            bottom_bound: f32::min(bottom_left.get_y(), top_right.get_y()),
            right_bound: f32::max(bottom_left.get_x(), top_right.get_x()),
            top_bound: f32::max(bottom_left.get_y(), top_right.get_y()),
        }
    }
}

impl<T: Clone + Fn(Point) -> Point + 'static, B: Clone + Fn(Point) -> Point + 'static> DrawnRegion
    for TransformedDrawnRegion<T, B>
{
    fn is_inside(&self, point: Point) -> bool {
        let transformed = (self.transform_function)(point);
        self.region.is_inside(transformed)
    }

    fn clone(&self) -> Box<dyn DrawnRegion> {
        Box::new(Self::new(
            self.region.clone(),
            self.transform_function.clone(),
            self.transform_back_function.clone(),
        ))
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
        let inner_intersection = self.region.find_line_intersection(
            (self.transform_function)(from),
            (self.transform_function)(to),
        );

        return match inner_intersection {
            LineIntersection::FullyInside => LineIntersection::FullyInside,
            LineIntersection::FullyOutside => LineIntersection::FullyOutside,
            LineIntersection::Enters { point } => LineIntersection::Enters {
                point: (self.transform_back_function)(point),
            },
            LineIntersection::Exits { point } => LineIntersection::Exits {
                point: (self.transform_back_function)(point),
            },
            LineIntersection::Crosses { entrance, exit } => LineIntersection::Crosses {
                entrance: (self.transform_back_function)(entrance),
                exit: (self.transform_back_function)(exit),
            },
        };
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn basic_test() {
        let original_region = Box::new(RectangularDrawnRegion::new(1.0, 4.0, 2.0, 7.0));
        let region = TransformedDrawnRegion::new(
            original_region,
            |point| Point::new(point.get_x() * 3.0, point.get_y() - 1.0),
            |point| Point::new(point.get_x() / 3.0, point.get_y() + 1.0),
        );
        assert!(!region.is_inside(Point::new(0.3, 4.5)));
        assert!(!region.is_inside(Point::new(0.4, 4.5)));
        assert!(region.is_inside(Point::new(0.4, 5.5)));
        assert!(region.is_inside(Point::new(0.65, 7.5)));
        assert!(!region.is_inside(Point::new(0.7, 7.5)));
        assert!(!region.is_inside(Point::new(0.7, 8.5)));

        assert_eq!(1.0 / 3.0, region.get_left());
        assert_eq!(5.0, region.get_bottom());
        assert_eq!(2.0 / 3.0, region.get_right());
        assert_eq!(8.0, region.get_top());
    }

    #[test]
    fn test_negated_bounds() {
        let transform_function = |point: Point| Point::new(-point.get_x(), -point.get_y());
        let simple_region = Box::new(RectangularDrawnRegion::new(0.0, 0.0, 1.0, 1.0));
        let transformed_region = TransformedDrawnRegion::new(
            simple_region,
            transform_function.clone(),
            transform_function.clone(),
        );
        assert_eq!(-1.0, transformed_region.get_left());
        assert_eq!(-1.0, transformed_region.get_bottom());
        assert_eq!(0.0, transformed_region.get_right());
        assert_eq!(0.0, transformed_region.get_top());
    }

    #[test]
    fn test_find_line_intersection() {
        let original_region = Box::new(RectangularDrawnRegion::new(0.0, 1.0, 3.0, 2.0));
        let transform_function =
            |point: Point| Point::new(5.0 * point.get_x() - 3.0, -4.0 * point.get_y() + 6.0);
        let transform_back_function =
            |point: Point| Point::new((point.get_x() + 3.0) / 5.0, (point.get_y() - 6.0) / -4.0);
        let transformed_region = TransformedDrawnRegion::new(
            original_region,
            transform_function,
            transform_back_function,
        );

        // The transformed region should be (left: 0.6, bottom: 1.25, right: 1.2, top: 1.0)
        assert!(LineIntersection::Crosses {
            entrance: Point::new(0.6, 1.1),
            exit: Point::new(1.2, 1.1)
        }
        .nearly_equal(
            transformed_region.find_line_intersection(Point::new(-2.0, 1.1), Point::new(20.0, 1.1))
        ));
    }
}
