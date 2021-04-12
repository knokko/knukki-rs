mod atlas;

pub use atlas::*;

use crate::Color;

use std::ops::{
    Index, IndexMut
};

#[derive(Clone)]
pub struct Texture {
    width: u32,
    height: u32,

    pixels: Vec<Color>,
}

impl Texture {
    pub fn new(width: u32, height: u32, background: Color) -> Self {
        let pixels = vec![background; (width * height) as usize];
        Self {
            width, height, pixels
        }
    }

    pub fn get_width(&self) -> u32{
        self.width
    }

    pub fn get_height(&self) -> u32{
        self.height
    }

    fn index(&self, x: u32, y: u32) -> usize {
        assert!(x < self.width);
        assert!(y < self.height);
        (x * self.height + y) as usize
    }

    pub fn get_color(&self, x: u32, y: u32) -> Color {
        self.pixels[self.index(x, y)]
    }

    pub fn set_color(&mut self, x: u32, y: u32, new_color: Color) {
        let index = self.index(x, y);
        self.pixels[index] = new_color;
    }

    pub fn fill_rect(&mut self, min_x: u32, min_y: u32, width: u32, height: u32, new_color: Color) {

        let bound_x = min_x + width;
        let bound_y = min_y + height;
        assert!(bound_x <= self.width);
        assert!(bound_y <= self.height);

        for x in min_x .. bound_x {
            for y in min_y .. bound_y {
                self[x][y as usize] = new_color;
            }
        }
    }

    pub fn copy_to(
        &self, own_min_x: u32, own_min_y: u32, copy_width: u32, copy_height: u32,
        destination: &mut Texture, dest_min_x: u32, dest_min_y: u32
    ) {
        assert!(own_min_x + copy_width <= self.width);
        assert!(own_min_y + copy_height <= self.height);
        assert!(dest_min_x + copy_width <= destination.width);
        assert!(dest_min_y + copy_height <= destination.height);

        for offset_x in 0 .. copy_width {
            for offset_y in 0 .. copy_height {
                let new_pixel = self[own_min_x + offset_x][(own_min_y + offset_y) as usize];
                destination[dest_min_x + offset_x][(dest_min_y + offset_y) as usize] = new_pixel;
            }
        }
    }

    pub fn copy_to_pixel_buffer(&self, dest: &mut [u8]) {
        for x in 0 .. self.width {
            for y in 0 .. self.height {
                let dest_index = 4 * (x + y * self.width) as usize;
                let source_color = self[x][y as usize];
                dest[dest_index] = source_color.get_red_int();
                dest[dest_index + 1] = source_color.get_green_int();
                dest[dest_index + 2] = source_color.get_blue_int();
                dest[dest_index + 3] = source_color.get_alpha_int();
            }
        }
    }

    pub fn create_pixel_buffer(&self) -> Vec<u8> {
        let mut pixel_buffer = vec![0; (self.width * self.height * 4) as usize];
        self.copy_to_pixel_buffer(&mut pixel_buffer);
        pixel_buffer
    }
}

impl Index<u32> for Texture {
    type Output = [Color];

    fn index(&self, column_index: u32) -> &Self::Output {
        let offset = (column_index * self.height) as usize;
        &self.pixels[offset .. offset + self.height as usize]
    }
}

impl IndexMut<u32> for Texture {
    fn index_mut(&mut self, column_index: u32) -> &mut Self::Output {
        let offset = (column_index * self.height) as usize;
        &mut self.pixels[offset .. offset + self.height as usize]
    }
}

#[cfg(test)]
mod tests {

    use crate::Color;

    use super::Texture;

    #[test]
    fn test_set_get_fill() {
        let background_color = Color::rgba(100, 0, 0, 200);
        let width = 3;
        let height = 5;

        let mut texture = Texture::new(width, height, background_color);
        assert_eq!(3, texture.get_width());
        assert_eq!(5, texture.get_height());
        assert_eq!(vec![background_color; 15], texture.pixels);

        for x in 0 .. width {
            for y in 0 .. height {
                assert_eq!(background_color, texture[x][y as usize]);
            }
        }

        let green = Color::rgb(0, 200, 0);
        let blue = Color::rgb(0, 0, 250);

        texture[2][4] = green;
        texture.set_color(1, 2, blue);

        assert_eq!(green, texture[2][4]);
        assert_eq!(green, texture.get_color(2, 4));
        assert_eq!(blue, texture[1][2]);
        assert_eq!(blue, texture.get_color(1, 2));

        assert_eq!(background_color, texture[2][2]);
        assert_eq!(background_color, texture.get_color(2, 2));

        let white = Color::rgba(200, 200, 200, 100);

        texture.fill_rect(0, 1, 2, 1, white);
        assert_eq!(background_color, texture[0][0]);
        assert_eq!(white, texture[0][1]);
        assert_eq!(white, texture.get_color(0, 1));
        assert_eq!(white, texture[1][1]);
        assert_eq!(blue, texture[1][2]);
        assert_eq!(green, texture[2][4]);
    }

    #[test]
    fn test_copy() {
        let red = Color::rgb(200, 0, 0);
        let green = Color::rgb(0, 200, 0);

        let source = Texture::new(4, 10, red);
        let mut destination = Texture::new(8, 3, green);

        source.copy_to(0, 4, 4, 1,
                       &mut destination, 2, 1
        );

        for x in 0 .. 8 {
            assert_eq!(green, destination[x][0]);
            assert_eq!(green, destination[x][2]);
        }

        for y in 0 .. 3 {
            assert_eq!(green, destination[0][y]);
            assert_eq!(green, destination[1][y]);
            assert_eq!(green, destination[6][y]);
            assert_eq!(green, destination[7][y]);
        }

        for x in 2 .. 6 {
            assert_eq!(red, destination[x][1]);
        }
    }

    #[test]
    fn test_pixel_buffer() {
        let color1 = Color::rgba(13, 87, 105, 255);
        let color2 = Color::rgb(217, 185, 197);
        let color3 = Color::rgba(201, 140, 0, 200);
        let color4 = Color::rgba(15, 97, 5, 0);
        let color5 = Color::rgb(89, 58, 240);

        let mut texture = Texture::new(2, 3, Color::rgb(200, 100, 150));
        texture[0][0] = color1;
        texture[1][0] = color2;
        texture[0][1] = color3;
        texture[1][1] = color4;
        texture[0][2] = color5;

        let pixel_buffer = texture.create_pixel_buffer();
        assert_eq!(vec![
            13, 87, 105, 255, 217, 185, 197, 255, 201, 140, 0, 200, 15, 97, 5, 0, 89, 58, 240, 255, 200, 100, 150, 255
        ], pixel_buffer);
    }
}
