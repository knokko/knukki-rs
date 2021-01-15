mod simple_flat;
mod hover_circle;

pub use simple_flat::*;
pub use hover_circle::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Color {
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: u8::max_value(),
        }
    }

    pub fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn get_red_int(&self) -> u8 {
        self.red
    }

    pub fn get_green_int(&self) -> u8 {
        self.green
    }

    pub fn get_blue_int(&self) -> u8 {
        self.blue
    }

    pub fn get_alpha_int(&self) -> u8 {
        self.alpha
    }

    pub fn get_red_float(&self) -> f32 {
        self.red as f32 / 255.0
    }

    pub fn get_green_float(&self) -> f32 {
        self.green as f32 / 255.0
    }

    pub fn get_blue_float(&self) -> f32 {
        self.blue as f32 / 255.0
    }

    pub fn get_alpha_float(&self) -> f32 {
        self.alpha as f32 / 255.0
    }
}
