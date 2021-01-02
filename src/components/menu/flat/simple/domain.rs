use crate::Point;

#[derive(Copy, Clone, Debug)]
pub struct ComponentDomain {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl ComponentDomain {
    pub fn between(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn with_size(min_x: f32, min_y: f32, width: f32, height: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x: min_x + width,
            max_y: min_y + height,
        }
    }

    pub fn get_min_x(&self) -> f32 {
        self.min_x
    }

    pub fn get_min_y(&self) -> f32 {
        self.min_y
    }

    pub fn get_max_x(&self) -> f32 {
        self.max_x
    }

    pub fn get_max_y(&self) -> f32 {
        self.max_y
    }

    pub fn get_width(&self) -> f32 {
        self.max_x - self.min_x
    }

    pub fn get_height(&self) -> f32 {
        self.max_y - self.min_y
    }

    pub fn is_inside(&self, point: Point) -> bool {
        point.get_x() >= self.get_min_x()
            && point.get_x() <= self.get_max_x()
            && point.get_y() >= self.get_min_y()
            && point.get_y() <= self.get_max_y()
    }

    pub fn transform(&self, outer: Point) -> Point {
        let inner_x = (outer.get_x() - self.get_min_x()) / self.get_width();
        let inner_y = (outer.get_y() - self.get_min_y()) / self.get_height();
        Point::new(inner_x, inner_y)
    }

    pub fn transform_back(&self, inner: Point) -> Point {
        let outer_x = self.get_min_x() + inner.get_x() * self.get_width();
        let outer_y = self.get_min_y() + inner.get_y() * self.get_height();
        Point::new(outer_x, outer_y)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_between() {
        // Carefully choose values to ensure they don't cause rounding errors
        let domain = ComponentDomain::between(-0.5, 0.25, 1.25, 0.5);
        assert_eq!(-0.5, domain.get_min_x());
        assert_eq!(0.25, domain.get_min_y());
        assert_eq!(1.25, domain.get_max_x());
        assert_eq!(0.5, domain.get_max_y());
        assert_eq!(1.75, domain.get_width());
        assert_eq!(0.25, domain.get_height());
    }

    #[test]
    fn test_with_size() {
        let domain = ComponentDomain::with_size(-0.75, -1.0, 0.5, 1.0);
        assert_eq!(-0.75, domain.get_min_x());
        assert_eq!(-1.0, domain.get_min_y());
        assert_eq!(-0.25, domain.get_max_x());
        assert_eq!(0.0, domain.get_max_y());
        assert_eq!(0.5, domain.get_width());
        assert_eq!(1.0, domain.get_height());
    }

    #[test]
    fn test_is_inside() {
        let domain = ComponentDomain::between(1.0, 0.0, 2.0, 3.0);

        assert!(!domain.is_inside(Point::new(-0.5, -0.5)));
        assert!(!domain.is_inside(Point::new(0.5, 0.5)));
        assert!(domain.is_inside(Point::new(1.5, 0.5)));
        assert!(!domain.is_inside(Point::new(1.5, 3.5)));
        assert!(!domain.is_inside(Point::new(2.5, 3.5)));

        // Edge case, literally
        assert!(domain.is_inside(Point::new(1.5, 0.0)));

        // Corner case, literally
        assert!(domain.is_inside(Point::new(2.0, 3.0)));
    }

    #[test]
    fn test_transform() {
        // These numbers are carefully chosen to avoid rounding errors
        let domain = ComponentDomain::between(0.25, 0.5, 0.375, 0.75);

        assert_eq!(
            Point::new(0.0, 0.0),
            domain.transform(Point::new(0.25, 0.5))
        );
        assert_eq!(
            Point::new(1.0, 1.0),
            domain.transform(Point::new(0.375, 0.75))
        );
        assert_eq!(
            Point::new(0.25, 0.5),
            domain.transform(Point::new(0.28125, 0.625))
        );
        assert_eq!(
            Point::new(-2.0, -2.0),
            domain.transform(Point::new(0.0, 0.0))
        );
        assert_eq!(Point::new(6.0, 2.0), domain.transform(Point::new(1.0, 1.0)));
    }

    #[test]
    fn test_transform_back() {
        // This is just the reverse of the test_transform test
        let domain = ComponentDomain::between(0.25, 0.5, 0.375, 0.75);

        assert_eq!(
            Point::new(0.25, 0.5),
            domain.transform_back(Point::new(0.0, 0.0))
        );
        assert_eq!(
            Point::new(0.375, 0.75),
            domain.transform_back(Point::new(1.0, 1.0))
        );
        assert_eq!(
            Point::new(0.28125, 0.625),
            domain.transform_back(Point::new(0.25, 0.5))
        );
        assert_eq!(
            Point::new(0.0, 0.0),
            domain.transform_back(Point::new(-2.0, -2.0))
        );
        assert_eq!(
            Point::new(1.0, 1.0),
            domain.transform_back(Point::new(6.0, 2.0))
        );
    }
}
