use crate::*;

mod manager;

pub use manager::*;
use font_kit::source::SystemSource;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::Vector2I;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::hinting::HintingOptions;
use font_kit::canvas::{RasterizationOptions, Canvas, Format};

pub trait Font {
    fn draw_grapheme(&self, grapheme: &str, point_size: f32) -> Texture;
}

pub struct SystemFont {

}

impl SystemFont {
    pub fn new() -> Self {
        Self {}
    }

    pub fn test() {
        let system_font_source = SystemSource::new();
        let mut all_fonts: Vec<font_kit::font::Font> = system_font_source.all_fonts()
            .expect("Should be able to list fonts")
            .into_iter().map(|handle| handle.load().unwrap()).collect();

        all_fonts.sort_by_key(|font| font.glyph_count());
        for font in all_fonts {
            println!("Font is {:?} and glyph count is {}", font, font.glyph_count());
            println!("Postscript name is {:?}", font.postscript_name());
        }
    }
}

impl Font for SystemFont {
    fn draw_grapheme(&self, grapheme: &str, point_size: f32) -> Texture {

        let system_font_source = SystemSource::new();
        // let font_handle = system_font_source.select_best_match(&[
        //     FamilyName::SansSerif
        // ], &Properties::new()).unwrap();
        let font_handle = system_font_source.select_by_postscript_name(
            "NotoSerifCJKtc-Regular"
        ).expect("Should have the font");

        let font = font_handle.load().unwrap();

        let hinting_options = HintingOptions::None;
        let rasterization_options = RasterizationOptions::GrayscaleAa;
        let canvas_format = Format::A8;

        // TODO Use graphemes instead
        let glyph_id = font.glyph_for_char(
            grapheme.chars().next().expect("At least 1 char was given")
        ).expect("Should have the glyph id for this character");

        let raster_rect = font.raster_bounds(
            glyph_id,
            point_size,
            Transform2F::default(),
            hinting_options,
            rasterization_options
        ).unwrap();

        let mut glyph_canvas = Canvas::new(raster_rect.size(), canvas_format);

        font.rasterize_glyph(
            &mut glyph_canvas,
            glyph_id,
            point_size,
            Transform2F::from_translation(-raster_rect.origin().to_f32()),
            hinting_options,
            rasterization_options,
        ).unwrap();

        let width = glyph_canvas.size.x() as u32;
        let height = glyph_canvas.size.y() as u32;
        let mut glyph_texture = Texture::new(
            width, height, Color::rgb(100, 200, 200)
        );

        for x in 0 .. width {
            for y in 0 .. height {
                let grayscale = glyph_canvas.pixels[(x + (height - y - 1) * width) as usize];
                glyph_texture[x][y as usize] = Color::rgb(grayscale, 0, 0);
            }
        }

        glyph_texture
    }
}