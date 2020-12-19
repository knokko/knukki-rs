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
}
