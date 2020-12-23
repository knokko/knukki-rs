use crate::MousePoint;

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

    pub fn is_inside(&self, x: f32, y: f32) -> bool {
        x >= self.get_min_x()
            && x <= self.get_max_x()
            && y >= self.get_min_y()
            && y <= self.get_max_y()
    }

    pub fn transform(&self, outer_x: f32, outer_y: f32) -> (f32, f32) {
        let inner_x = (outer_x - self.get_min_x()) / self.get_width();
        let inner_y = (outer_y - self.get_min_y()) / self.get_height();
        (inner_x, inner_y)
    }

    pub fn transform_mouse(&self, mouse_point: MousePoint) -> MousePoint {
        let transformed = self.transform(mouse_point.get_x(), mouse_point.get_y());
        MousePoint::new(transformed.0, transformed.1)
    }

    pub fn transform_back(&self, inner_x: f32, inner_y: f32) -> (f32, f32) {
        let outer_x = self.get_min_x() + inner_x * self.get_width();
        let outer_y = self.get_min_y() + inner_y * self.get_height();
        (outer_x, outer_y)
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

        assert!(!domain.is_inside(-0.5, -0.5));
        assert!(!domain.is_inside(0.5, 0.5));
        assert!(domain.is_inside(1.5, 0.5));
        assert!(!domain.is_inside(1.5, 3.5));
        assert!(!domain.is_inside(2.5, 3.5));

        // Edge case, literally
        assert!(domain.is_inside(1.5, 0.0));

        // Corner case, literally
        assert!(domain.is_inside(2.0, 3.0));
    }

    #[test]
    fn test_transform() {
        // These numbers are carefully chosen to avoid rounding errors
        let domain = ComponentDomain::between(0.25, 0.5, 0.375, 0.75);

        assert_eq!((0.0, 0.0), domain.transform(0.25, 0.5));
        assert_eq!((1.0, 1.0), domain.transform(0.375, 0.75));
        assert_eq!((0.25, 0.5), domain.transform(0.28125, 0.625));
        assert_eq!((-2.0, -2.0), domain.transform(0.0, 0.0));
        assert_eq!((6.0, 2.0), domain.transform(1.0, 1.0));
    }

    #[test]
    fn test_transform_back() {
        // This is just the reverse of the test_transform test
        let domain = ComponentDomain::between(0.25, 0.5, 0.375, 0.75);

        assert_eq!((0.25, 0.5), domain.transform_back(0.0, 0.0));
        assert_eq!((0.375, 0.75), domain.transform_back(1.0, 1.0));
        assert_eq!((0.28125, 0.625), domain.transform_back(0.25, 0.5));
        assert_eq!((0.0, 0.0), domain.transform_back(-2.0, -2.0));
        assert_eq!((1.0, 1.0), domain.transform_back(6.0, 2.0));
    }
}
