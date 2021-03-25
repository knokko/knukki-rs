mod atlas;

pub use atlas::*;

use crate::Color;

use std::ops::{
    Index, IndexMut
};

#[derive(Clone)]
pub struct Texture {
    width: usize,
    height: usize,

    pixels: Vec<Color>,
}

impl Texture {
    pub fn new(width: usize, height: usize, background: Color) -> Self {
        let pixels = vec![background; width * height];
        Self {
            width, height, pixels
        }
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    fn index(&self, x: usize, y: usize) -> usize {
        assert!(x < self.width);
        assert!(y < self.height);
        x * self.height + y
    }

    pub fn get_color(&self, x: usize, y: usize) -> Color {
        self.pixels[self.index(x, y)]
    }

    pub fn set_color(&mut self, x: usize, y: usize, new_color: Color) {
        let index = self.index(x, y);
        self.pixels[index] = new_color;
    }

    pub fn fill_rect(&mut self, min_x: usize, min_y: usize, width: usize, height: usize, new_color: Color) {

        let bound_x = min_x + width;
        let bound_y = min_y + height;
        assert!(bound_x <= self.width);
        assert!(bound_y <= self.height);

        for x in min_x .. bound_x {
            for y in min_y .. bound_y {
                self[x][y] = new_color;
            }
        }
    }

    pub fn copy_to(
        &self, own_min_x: usize, own_min_y: usize, copy_width: usize, copy_height: usize,
        destination: &mut Texture, dest_min_x: usize, dest_min_y: usize
    ) {
        assert!(own_min_x + copy_width <= self.width);
        assert!(own_min_y + copy_height <= self.height);
        assert!(dest_min_x + copy_width <= destination.width);
        assert!(dest_min_y + copy_height <= destination.height);

        for offset_x in 0 .. copy_width {
            for offset_y in 0 .. copy_height {
                let new_pixel = self[own_min_x + offset_x][own_min_y + offset_y];
                destination[dest_min_x + offset_x][dest_min_y + offset_y] = new_pixel;
            }
        }
    }
}

impl<'a> Index<usize> for Texture {
    type Output = [Color];

    fn index(&self, column_index: usize) -> &Self::Output {
        let offset = column_index * self.height;
        &self.pixels[offset .. offset + self.height]
    }
}

impl<'a> IndexMut<usize> for Texture {
    fn index_mut(&mut self, column_index: usize) -> &mut Self::Output {
        let offset = column_index * self.height;
        &mut self.pixels[offset .. offset + self.height]
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
                assert_eq!(background_color, texture[x][y]);
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
}
