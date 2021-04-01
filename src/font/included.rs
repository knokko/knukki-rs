use ab_glyph::{FontRef, Font, InvalidFont, OutlinedGlyph};
use crate::Texture;
use unicode_segmentation::UnicodeSegmentation;

/*
 * We COULD use this in WebAssembly as well, but it would add ~12MB to the wasm file. Without this,
 * the wasm file would be ~150KB. Since loading time is crucial for webpages, this would be a
 * disaster. Instead, I will use Canvas2D API on an offscreen canvas.
 *
 * Including this file in desktop targets will also increase the binary size of those desktop
 * releases by ~12 MB. But, this is not a big problem because these releases are already 50+MB
 * even without this file (the binary would increase by a factor of ~1.2, which is somewhat
 * significant, but nothing like the factor of ~75 for WebAssembly. Besides, we don't have a
 * Canvas2D API on desktop targets, so we don't have much choice. (We could try to work with
 * system fonts, but these are not so nice to work with.)
 */
#[cfg(not(target_arch = "wasm32"))]
pub fn create_default_font() -> IncludedStaticFont {
    IncludedStaticFont::new(include_bytes!("Code2003-W8nn.ttf")).expect("Unifont is valid")
}

pub struct IncludedStaticFont {
    internal_font: FontRef<'static>
}

impl IncludedStaticFont {
    pub fn new(raw_data: &'static [u8]) -> Result<Self, InvalidFont> {
        let internal_font = FontRef::try_from_slice(raw_data)?;
        Ok(Self {
            internal_font
        })
    }
}

impl crate::Font for IncludedStaticFont {
    fn draw_grapheme(&self, grapheme: &str, point_size: f32) -> Option<Texture> {

        let all_outlines: Vec<_> = grapheme.chars().map(|current_char| {
            if !current_char.is_whitespace() {
                let current_glyph_id = self.internal_font.glyph_id(current_char);
                let current_glyph = current_glyph_id.with_scale(point_size);
                // TODO Supply fallback texture
                Some(self.internal_font.outline_glyph(current_glyph)
                    .expect("Should be able to outline the current glyph"))
            } else {
                None
            }

        }).collect();

        if all_outlines.is_empty() {
            panic!("Not a single character was supplied");
        }

        struct CharOutline {
            outline: OutlinedGlyph,
            offset_x: i32,
            offset_y: i32,
        }

        let mut combined_min_x = i32::max_value();
        let mut combined_min_y = i32::max_value();
        let mut combined_max_x = i32::min_value();
        let mut combined_max_y = i32::min_value();

        let mut char_index = 0;
        let detailed_outlines: Vec<_> = all_outlines.into_iter().map(|maybe_current_outline| {

            let mapped = if let Some(current_outline) = maybe_current_outline {
                let mut min_x = i32::max_value();
                let mut min_y = i32::max_value();
                let mut max_x = i32::min_value();
                let mut max_y = i32::min_value();
                current_outline.draw(|x, y, _value| {
                    min_x = min_x.min(x as i32);
                    min_y = min_y.min(y as i32);
                    max_x = max_x.max(x as i32);
                    max_y = max_y.max(y as i32);
                });

                let offset_x = 0;
                let offset_y = current_outline.px_bounds().min.y as i32;

                // Potential edge case for weird whitespace characters
                if min_x != i32::max_value() {
                    min_x += offset_x;
                    min_y += offset_y;
                    max_x += offset_x;
                    max_y += offset_y;
                }

                combined_min_x = combined_min_x.min(min_x);
                combined_min_y = combined_min_y.min(min_y);
                combined_max_x = combined_max_x.max(max_x);
                combined_max_y = combined_max_y.max(max_y);

                Some(CharOutline {
                    outline: current_outline,
                    offset_x, offset_y
                })
            } else {
                None
            };

            char_index += 1;
            mapped
        }).collect();

        // If we only got whitespace characters, we should return None
        if combined_min_x == i32::max_value() {
            return None;
        }

        let combined_offset_x = -combined_min_x;
        let combined_offset_y = -combined_min_y;
        let width = (combined_max_x - combined_min_x + 1) as u32;
        let height = (combined_max_y - combined_min_y + 1) as u32;

        let mut grayscale = vec![0.0; (width * height) as usize];

        for maybe_detailed_outline in &detailed_outlines {
            if let Some(detailed_outline) = maybe_detailed_outline {
                let current_outline = &detailed_outline.outline;
                current_outline.draw(|relative_x, relative_y, value| {
                    let x = (relative_x as i32 + detailed_outline.offset_x + combined_offset_x) as u32;
                    let y = (relative_y as i32 + detailed_outline.offset_y + combined_offset_y) as u32;
                    let index = (x + y * width) as usize;
                    if value > grayscale[index] {
                        grayscale[index] = value;
                    }
                });
            }
        }

        let mut texture = Texture::new(width, height, crate::Color::rgb(0, 0, 0));
        for index in 0 .. grayscale.len() {
            let width = width as usize;
            let x = (index % width) as u32;
            let y = height as usize - index / width - 1;
            let int_value = (grayscale[index] * 255.0) as u8;
            let color = crate::Color::rgb(int_value, 0, 0);
            texture[x][y] = color;
        }

        Some(texture)
    }
}