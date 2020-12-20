use crate::MousePoint;

#[derive(Copy, Clone, Debug)]
pub struct ComponentDomain {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32
}

impl ComponentDomain {
    pub fn between(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self { min_x, min_y, max_x, max_y }
    }

    pub fn with_size(min_x: f32, min_y: f32, width: f32, height: f32) -> Self {
        Self { min_x, min_y, max_x: min_x + width, max_y: min_y + height }
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
        x >= self.get_min_x() && x <= self.get_max_x() 
        && y >= self.get_min_y() && y <= self.get_max_y()
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
}
