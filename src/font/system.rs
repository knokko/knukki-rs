// NOTE: THIS MODULE IS NOT FINISHED AT ALL
use crate::*;

use font_kit::source::SystemSource;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::Vector2I;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::hinting::HintingOptions;
use font_kit::canvas::{RasterizationOptions, Canvas, Format};
use unicode_segmentation::*;
use ttf_parser::Face;
use font_kit::family_handle::FamilyHandle;

pub struct SystemFont {

}

impl SystemFont {
    pub fn new() -> Self {
        Self {}
    }

    pub fn test() {
        let system_font_source = SystemSource::new();
        // let mut all_fonts: Vec<font_kit::font::Font> = system_font_source.all_fonts()
        //     .expect("Should be able to list fonts")
        //     .into_iter().map(|handle| handle.load().unwrap()).collect();
        //
        // all_fonts.sort_by_key(|font| font.glyph_count());
        // for font in all_fonts {
        //     println!("Font is {:?} and glyph count is {}", font, font.glyph_count());
        //     println!("Postscript name is {:?}", font.postscript_name());
        // }

        struct FontFamily {
            name: String,
            handle: FamilyHandle
        }

        impl FontFamily {
            fn glyph_count(&self) -> u32 {
                let mut count = 0;
                for font_handle in self.handle.fonts() {
                    let font = font_handle.load().expect("Should be able to load all fonts");
                    count += font.glyph_count();
                }

                count
            }
        }

        let mut all_families: Vec<_> = system_font_source.all_families()
            .expect("Should be able to list font families")
            .into_iter().map(|name|
                FontFamily { handle: system_font_source.select_family_by_name(&name)
                .expect("Should be able to get all family handles"), name })
            .collect();

        all_families.sort_by_key(|family| family.glyph_count());
        all_families.reverse();

        for family in all_families {
            println!("Font family {}: ({})", family.name, family.glyph_count());
            for font_handle in family.handle.fonts() {
                let font = font_handle.load().expect("Should be able to load all fonts");
                println!("{:?} ({}) and glyph count is {}", font.postscript_name(), font.full_name(), font.glyph_count());
            }
            println!();
        }

        println!("Best match searches:");
        for family_name in &[FamilyName::Serif, FamilyName::SansSerif, FamilyName::Cursive,
                FamilyName::Fantasy, FamilyName::Monospace] {
            let result = system_font_source.select_best_match(
                &[family_name.clone()], &Properties::default()
            );
            println!("{:?} found {:?}", family_name, result);
        }
    }
}

impl Font for SystemFont {
    fn draw_grapheme(&self, grapheme: &str, point_size: f32) -> Option<CharTexture> {

        let system_font_source = SystemSource::new();
        // let font_handle = system_font_source.select_best_match(&[
        //     FamilyName::SansSerif
        // ], &Properties::new()).unwrap();
        let font_handle = system_font_source.select_by_postscript_name(
            "DroidSansFallback"
        ).expect("Should have the font");

        let font = font_handle.load().unwrap();

        let hinting_options = HintingOptions::None;
        let rasterization_options = RasterizationOptions::GrayscaleAa;
        let canvas_format = Format::A8;

        // TODO Use graphemes instead
        //UnicodeSegmentation::graphemes();
        let char_face = Face::from_slice(grapheme.as_bytes(), 0);
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

        //Some(glyph_texture)
        todo!()
    }

    fn get_max_descent(&self, point_size: f32) -> f32 {
        unimplemented!()
    }

    fn get_max_ascent(&self, point_size: f32) -> f32 {
        unimplemented!()
    }

    fn get_whitespace_width(&self, point_size: f32) -> f32 {
        unimplemented!()
    }
}