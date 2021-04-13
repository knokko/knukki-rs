mod error;
mod group;
mod position;

pub use error::*;
pub use group::*;
pub use position::*;

use crate::*;

use std::cell::Cell;
use std::rc::Rc;

/// Represents a texture atlas. This is a big texture on which many smaller textures are stored.
/// Texture atlases are rarely drawn *entirely* (that is only for debugging purposes). Instead, the
/// smaller textures *on* the atlas are drawn.
///
/// ## Motivation
/// The advantage of using texture atlases rather than many small textures is that small textures on
/// the same atlas can be drawn in parallel. This is useful if many of the small textures are
/// related to each other. The most important use case (since this is a GUI crate after all) is text
/// rendering: each character of a given alphabet could be stored on the same atlas, which would
/// allow entire paragraphs of text to be drawn in parallel. Texture atlases are also useful for
/// drawing game worlds (typically, the same tiles are used a lot, so many tiles can be drawn in
/// parallel if their textures are on the same atlas).
///
/// ## Group
/// Texture atlases have limited space (most GPU's don't support huge textures). To overcome this
/// limitation (at least partially), the `TextureAtlasGroup` struct can be used instead (it will use
/// multiple `TextureAtlas`es internally). Such a group will also make it easier to deal with
/// texture replacements.
pub struct TextureAtlas {
    big_texture: Texture,

    placements: Vec<Rc<PlacedTexture>>,
    rows_info: RowsInfo,
}

impl TextureAtlas {
    /// Constructs and returns a new empty `TextureAtlas` width the given `width` and `height`
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            // We use a very weird background color (pink) because it should never be shown and it
            // will speed up debugging if it is shown for some reason
            big_texture: Texture::new(width, height, Color::rgb(200, 0, 100)),

            placements: Vec::new(),
            rows_info: RowsInfo::new(width, height),
        }
    }

    /// Gets a reference to the texture on which all textures are placed
    pub fn get_texture(&self) -> &Texture {
        &self.big_texture
    }

    /// Attempts to place the given `textures` onto this texture atlas.
    ///
    /// ## Procedure
    /// First, this method will attempt to place the given textures on the unused space of this
    /// texture atlas. (This can fail if there is not enough place available or the place that is
    /// available is too fragmented.)
    ///
    /// If not all textures were placed on the unused space, this method will remove 'unimportant'
    /// existing textures to make space for the new textures. Textures are considered 'unimportant'
    /// when they haven't been used for a while or are not frequently used.
    ///
    /// If not all textures can be placed, even after removing all existing textures, some of the
    /// textures won't be placed.
    ///
    /// ## Return value
    /// The return value of this method is a `TexturePlaceResult`. It indicates which of the
    /// `textures` were placed successfully, where they were placed, and how many existing textures
    /// had to be sacrificed to make this happen. See the documentation of `TexturePlaceResult` for
    /// more information.
    ///
    /// ## Test
    /// The `test` parameter can be used to test/simulate this method. When you give it the value
    /// `true`, this method will return the same return value as if you used the value `false`, but
    /// without actually placing the textures (and without removing any textures on this atlas).
    /// This is particularly useful for `TextureAtlasGroup` to decide on which texture atlas to put
    /// a slice of textures (to avoid cases where not all textures can be placed on the same atlas
    /// or avoid removing existing textures).
    pub fn add_textures(&mut self, textures: &[&Texture], test: bool) -> TexturePlaceResult {

        let mut num_row_ratings = 0;
        let row_ratings: Vec<Vec<RowRating>> = textures.iter().map(|texture| {
            let ratings = self.rows_info.rank_placement_rows(texture.width, texture.height);
            num_row_ratings += ratings.len();
            ratings
        }).collect();

        let mut combined_ratings = Vec::with_capacity(num_row_ratings);
        for index in 0 .. row_ratings.len() {
            for row_rating in &row_ratings[index] {
                combined_ratings.push(IndexedRowRating { index, row_rating: *row_rating });
            }
        }

        combined_ratings.sort_unstable_by(|a, b| {
            a.row_rating.rating.partial_cmp(&b.row_rating.rating).expect("NaN is impossible")
        });
        combined_ratings.reverse();

        // It is time to find placement locations for the textures (but don't commit anything yet)
        let mut placements = vec![None; textures.len()];
        let mut test_rows_info = self.rows_info.clone();

        // First try to put some of the textures in existing rows in the atlas
        Self::place_in_existing_rows(
            &mut test_rows_info, &mut placements,
            textures, &combined_ratings
        );

        // Try to place the remaining textures in new rows
        Self::place_in_new_rows(
            &mut test_rows_info, &mut placements, textures
        );

        // TODO Create a mechanism to remove old textures

        // Unless this method call was a test, we should actually place these textures
        if !test {
            self.rows_info = test_rows_info;
        }

        let mut resulting_placements = Vec::with_capacity(placements.len());
        for index in 0 .. placements.len() {
            if let Some(position) = placements[index] {
                let placement = Rc::new(PlacedTexture {
                    position: Cell::new(Some(position)),
                    priority: Cell::new(PlacedTexture::INITIAL_PRIORITY)
                });

                if !test {
                    self.placements.push(Rc::clone(&placement));
                    textures[index].copy_to(
                        0, 0, position.width, position.height,
                        &mut self.big_texture, position.min_x, position.min_y
                    );
                }

                resulting_placements.push(placement);
            } else {
                resulting_placements.push(Rc::new(PlacedTexture {
                    position: Cell::new(None),
                    priority: Cell::new(0),
                }));
            }
        }

        TexturePlaceResult {
            placements: resulting_placements,
            num_replaced_textures: 0,
        }
    }

    fn place_in_existing_rows(
        rows_info: &mut RowsInfo, placements: &mut [Option<TextureAtlasPosition>],
        textures: &[&Texture], suggestions: &[IndexedRowRating]
    ) {

        for suggestion in suggestions {
            if placements[suggestion.index].is_none() {

                let row = &mut rows_info.rows[suggestion.row_rating.row_index];
                let width = textures[suggestion.index].get_width();
                if row.bound_x + width <= rows_info.atlas_width {

                    placements[suggestion.index] = Some(TextureAtlasPosition {
                        min_x: row.bound_x,
                        min_y: row.min_y,
                        width,
                        height: textures[suggestion.index].height,
                    });
                    row.bound_x += width;
                }
            }
        }
    }

    fn place_in_new_rows(
        rows_info: &mut RowsInfo, placements: &mut [Option<TextureAtlasPosition>],
        textures: &[&Texture]
    ) {

        struct RemainingTexture<'a> {
            texture: &'a Texture,
            index: usize,
        }

        let num_remaining_textures = placements.iter()
            .filter(|placement| placement.is_none()).count();
        let mut remaining_textures = Vec::with_capacity(num_remaining_textures);
        for index in 0 .. placements.len() {
            if placements[index].is_none() {
                remaining_textures.push(RemainingTexture { texture: textures[index], index });
            }
        }

        remaining_textures.sort_unstable_by_key(|texture| texture.texture.get_height());
        remaining_textures.reverse();

        for indexed_texture in remaining_textures {
            let texture = indexed_texture.texture;

            // Whether this texture is the first in a new row
            let add_new_row = match rows_info.rows.last() {
                Some(last_row) =>
                    (last_row.bound_x + texture.width > rows_info.atlas_width)
                        || (texture.height > last_row.height
                    ),
                None => true
            };

            if add_new_row {
                if rows_info.bound_y + texture.height <= rows_info.atlas_height {
                    rows_info.rows.push(RowInfo {
                        min_y: rows_info.bound_y,
                        height: texture.height,
                        bound_x: 0
                    });
                    rows_info.bound_y += texture.height;
                } else {
                    // When this occurs, the current texture can't be placed in a new row
                    continue;
                }
            }

            let dest_row = rows_info.rows.last_mut().unwrap();

            // Handle the edge case where the texture is wider than the texture atlas
            // And with handling, I mean simply not placing it (because it is impossible)
            if texture.width <= rows_info.atlas_width {
                placements[indexed_texture.index] = Some(TextureAtlasPosition {
                    min_x: dest_row.bound_x,
                    min_y: dest_row.min_y,
                    width: texture.width,
                    height: texture.height
                });
                dest_row.bound_x += texture.width;
            }
        }
    }
}

struct IndexedRowRating {
    index: usize,
    row_rating: RowRating,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct RowInfo {
    min_y: u32,
    height: u32,
    bound_x: u32,
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct RowsInfo {
    rows: Vec<RowInfo>,
    atlas_width: u32,
    atlas_height: u32,
    bound_y: u32,
}

impl RowsInfo {
    fn new(atlas_width: u32, atlas_height: u32) -> Self {
        Self {
            rows: Vec::new(), atlas_width, atlas_height, bound_y: 0
        }
    }

    fn rank_placement_rows(&self, texture_width: u32, texture_height: u32) -> Vec<RowRating> {
        let mut result = Vec::new();
        for index in 0 .. self.rows.len() {
            let row = self.rows[index];
            if row.height >= texture_height {
                // TODO Also allow replacing textures that are guaranteed to be unused
                if row.bound_x + texture_width <= self.atlas_width {
                    let rating = texture_height as f32 / row.height as f32;
                    result.push(RowRating { row_index: index, rating });
                }
            }
        }
        result
    }
}

#[derive(Copy, Clone, Debug)]
struct RowRating {
    row_index: usize,
    rating: f32,
}

/// The result type for the `add_textures` method of `TextureAtlas`. This indicates how many of
/// the given textures were successfully placed, where these textures were placed, and how many
/// existing textures had to be 'sacrificed' to make place for the new textures.
///
/// ## Placements
/// The `placements` field of this struct indicates where the `textures` were placed, if they were
/// placed. It will be a `Vec` with the same length as `textures`, and for each index 0 <= i <
/// `textures.len()`, `placements[i]` denotes the placement information of `textures[i]`. If the
/// `position` of a placement is `None`, the corresponding texture was *not* placed on the atlas.
///
/// ## Number of replaced textures
/// The `num_replaced_textures` field indicates how many existing textures on the atlas were
/// sacrificed to make place for the new `textures`. This will always be 0 as long as there is
/// enough place on the texture atlas, but existing textures will have to be removed if there is
/// *not* enough place for all.
pub struct TexturePlaceResult {
    pub placements: Vec<Rc<PlacedTexture>>,
    pub num_replaced_textures: u32,
}

pub struct PlacedTexture {
    position: Cell<Option<TextureAtlasPosition>>,

    // TODO Manage the priority somehow (for instance, increment each time it is used, and
    // periodically divide all priorities by 2)
    priority: Cell<u32>,
}

impl PlacedTexture {
    const INITIAL_PRIORITY: u32 = 10_000;

    /// Checks whether the texture is still present on the texture atlas at its original position.
    /// If this method returns `false`, the texture should be placed on the atlas again, and all
    /// models that used the texture should be recreated with the new texture position.
    pub fn is_valid(&self) -> bool {
        self.position.get().is_some()
    }

    /// Marks this placed texture as *invalid*. This should be done when the texture on the atlas
    /// is overwritten by another texture (or the atlas itself is removed).
    pub fn invalidate(&self) {
        self.position.set(None);
    }

    /// Gets the position of the texture on the atlas. If this placed texture is no longer valid,
    /// this will return `None`.
    pub fn get_position(&self) -> Option<TextureAtlasPosition> {
        self.position.get()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // TODO Test place_textures removal behavior

    fn assert_filled(atlas: &TextureAtlas, x: u32, y: u32, width: u32, height: u32, color: Color) {
        for test_x in x .. x + width {
            for test_y in y .. y + height {
                assert_eq!(color, atlas.get_texture()[test_x][test_y as usize]);
            }
        }
    }

    fn assert_result(
        positions: Vec<Option<TextureAtlasPosition>>, num_unplaced_textures: u32,
        result: TexturePlaceResult
    ) {
        assert_eq!(num_unplaced_textures, result.num_replaced_textures);
        assert_eq!(positions.len(), result.placements.len());

        for index in 0 .. positions.len() {
            assert_eq!(positions[index], result.placements[index].position.get());
        }
    }

    #[test]
    fn test_place_textures_one_by_one_unsorted() {
        let red = Color::rgb(200, 0, 0);
        let green = Color::rgb(0, 200, 0);
        let blue = Color::rgb(0, 0, 200);

        let mut atlas = TextureAtlas::new(25, 100);
        let old_color = atlas.get_texture()[0][0];
        let texture1 = Texture::new(10, 8, red);
        let texture2 = Texture::new(9, 12, green);
        let texture3 = Texture::new(15, 4, blue);

        assert_result(
            vec![Some(TextureAtlasPosition {
                min_x: 0, min_y: 0, width: 10, height: 8
            })], 0,
            atlas.add_textures(&[&texture1], false)
        );
        assert_filled(&atlas, 0, 0, 10, 8, red);

        assert_result(
            vec![Some(TextureAtlasPosition {
                min_x: 0, min_y: 8, width: 9, height: 12
            })], 0,
            atlas.add_textures(&[&texture2], true)
        );
        assert_filled(&atlas, 0, 8, 9, 12, old_color);

        assert_result(
            vec![Some(TextureAtlasPosition {
                min_x: 0, min_y: 8, width: 9, height: 12
            })], 0,
            atlas.add_textures(&[&texture2], false)
        );
        assert_filled(&atlas, 0, 8, 9, 12, green);

        assert_result(
            vec![Some(TextureAtlasPosition {
                min_x: 10, min_y: 0, width: 15, height: 4
            })], 0,
            atlas.add_textures(&[&texture3], false)
        );
        assert_filled(&atlas, 10, 0, 15, 4, blue);

        assert_filled(&atlas, 0, 20,
                      atlas.get_texture().width, atlas.get_texture().height - 20,
                      old_color);
    }

    #[test]
    fn test_place_textures_one_by_one_sorted() {
        let red = Color::rgb(200, 0, 0);
        let green = Color::rgb(0, 200, 0);
        let blue = Color::rgb(0, 0, 200);

        let mut atlas = TextureAtlas::new(25, 100);
        let old_color = atlas.get_texture()[0][0];
        let texture1 = Texture::new(10, 8, red);
        let texture2 = Texture::new(9, 12, green);
        let texture3 = Texture::new(15, 4, blue);

        assert_result(
            vec![Some(TextureAtlasPosition {
                min_x: 0, min_y: 0, width: 9, height: 12
            })], 0,
            atlas.add_textures(&[&texture2], false)
        );
        assert_filled(&atlas, 0, 0, 9, 12, green);

        assert_result(
            vec![Some(TextureAtlasPosition {
                min_x: 9, min_y: 0, width: 10, height: 8
            })], 0,
            atlas.add_textures(&[&texture1], false)
        );
        assert_filled(&atlas, 9, 0, 10, 8, red);

        assert_result(
            vec![Some(TextureAtlasPosition {
                min_x:0, min_y: 12, width: 15, height: 4
            })], 0,
            atlas.add_textures(&[&texture3], true)
        );
        assert_filled(&atlas, 0, 12, 15, 4, old_color);

        assert_result(
            vec![Some(TextureAtlasPosition {
                min_x:0, min_y: 12, width: 15, height: 4
            })], 0,
            atlas.add_textures(&[&texture3], false)
        );
        assert_filled(&atlas, 0, 12, 15, 4, blue);
    }

    #[test]
    fn test_place_textures_multiple() {
        let mut atlas = TextureAtlas::new(50, 18);
        let color1 = Color::rgb(87, 14, 108);
        let color2 = Color::rgb(5, 208, 190);
        let color3 = Color::rgb(200, 100, 150);
        let color4 = Color::rgb(134, 86, 0);
        let color5 = Color::rgb(201, 243, 129);

        let texture1 = Texture::new(20, 8, color1);
        let texture2 = Texture::new(21, 7, color2);
        let texture3 = Texture::new(30, 6, color3);
        let texture4 = Texture::new(20, 5, color4);
        let texture5 = Texture::new(10, 4, color5);

        assert_result(vec![
            Some(TextureAtlasPosition { min_x: 30, min_y: 8, width: 20, height: 5 }),
            Some(TextureAtlasPosition { min_x: 20, min_y: 0, width: 21, height: 7 }),
            Some(TextureAtlasPosition { min_x: 0, min_y: 0, width: 20, height: 8 }),
            Some(TextureAtlasPosition { min_x: 0, min_y: 14, width: 10, height: 4 }),
            Some(TextureAtlasPosition { min_x: 0, min_y: 8, width: 30, height: 6 })
        ], 0, atlas.add_textures(
            &[&texture4, &texture2, &texture1, &texture5, &texture3], false
        ));

        let color6 = Color::rgb(74, 183, 76);
        let color7 = Color::rgb(31, 108, 93);

        let texture6 = Texture::new(5, 7, color6);
        let texture7 = Texture::new(8, 4, color7);

        for test in &[true, false] {
            assert_result(vec![
                Some(TextureAtlasPosition { min_x: 10, min_y: 14, width: 8, height: 4 }),
                Some(TextureAtlasPosition { min_x: 41, min_y: 0, width: 5, height: 7 })
            ], 0, atlas.add_textures(&[&texture7, &texture6], *test));
        }

        let color8 = Color::rgb(241, 178, 250);
        let color9 = Color::rgb(157, 201, 37);

        let texture8 = Texture::new(4, 8, color8);
        let texture9 = Texture::new(12, 1, color9);

        assert_result(vec![
            Some(TextureAtlasPosition { min_x: 46, min_y: 0, width: 4, height: 8 }),
            Some(TextureAtlasPosition { min_x: 18, min_y: 14, width: 12, height: 1 })
        ], 0, atlas.add_textures(&[&texture8, &texture9], false));

        assert_filled(&atlas, 0, 0, 20, 8, color1);
        assert_filled(&atlas, 20, 0, 21, 7, color2);
        assert_filled(&atlas, 0, 08, 30, 6, color3);
        assert_filled(&atlas, 30, 8, 20, 5, color4);
        assert_filled(&atlas, 0, 14, 10, 4, color5);
        assert_filled(&atlas, 41, 0, 5, 7, color6);
        assert_filled(&atlas, 10, 14, 8, 4, color7);
        assert_filled(&atlas, 46, 0, 4, 8, color8);
        assert_filled(&atlas, 18, 14, 12, 1, color9);
    }

    #[test]
    fn test_place_textures_too_big() {
        let mut atlas = TextureAtlas::new(10, 10);
        let color = Color::rgb(1, 2, 3);

        assert_result(vec![None], 0, atlas.add_textures(
            &[&Texture::new(11, 10, color)], true
        ));
        assert_result(vec![None], 0, atlas.add_textures(
            &[&Texture::new(10, 11, color)], false
        ));
        assert_result(vec![None], 0, atlas.add_textures(
            &[&Texture::new(11, 11, color)], false
        ));
        assert_result(vec![Some(TextureAtlasPosition {
            min_x: 0, min_y: 0, width: 10, height: 10
        })], 0, atlas.add_textures(
            &[&Texture::new(10, 10, color)], true
        ));

        assert_result(vec![Some(TextureAtlasPosition {
            min_x: 0, min_y: 0, width: 1, height: 1
        })], 0, atlas.add_textures(&[&Texture::new(
            1, 1, color
        )], false));

        // The 10x10 won't fit anymore
        assert_result(vec![None], 0, atlas.add_textures(
            &[&Texture::new(10, 10, color)], false
        ));

        // Due to the algorithm implementation, 9x10 won't fit either
        assert_result(vec![None], 0, atlas.add_textures(
            &[&Texture::new(9, 10, color)], false
        ));

        // But 10x9 should still fit
        assert_result(vec![Some(TextureAtlasPosition {
            min_x: 0, min_y: 1, width: 10, height: 9
        })], 0, atlas.add_textures(&[&Texture::new(
            10, 9, color
        )], true));
    }
}