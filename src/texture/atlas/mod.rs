mod group;
mod id;
mod position;

pub use group::*;
pub use id::*;
pub use position::*;

use crate::*;

use std::cell::Cell;
use std::cmp::Ordering;
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

    next_id: u64,
    placements: Vec<Rc<PlacedTexture>>,
    rows_info: RowsInfo,
}

impl TextureAtlas {
    /// Constructs and returns a new empty `TextureAtlas` width the given `width` and `height`
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            // We use a very weird background color (pink) because it should never be shown and it
            // will speed up debugging if it is shown for some reason
            big_texture: Texture::new(width, height, Color::rgb(200, 0, 100)),

            next_id: 0,
            placements: Vec::new(),
            rows_info: RowsInfo::new(width),
        }
    }

    pub fn add_textures(&mut self, textures: Vec<&Texture>, test: bool) -> TexturePlaceResult {
        let mut num_row_ratings = 0;
        let row_ratings: Vec<Vec<RowRating>> = textures.iter().map(|texture| {
            let ratings = self.rows_info.rank_placement_rows(texture.width, texture.height);
            num_row_ratings += ratings.len();
            ratings
        }).collect();

        struct IndexedRowRating {
            index: usize,
            row_rating: RowRating,
        }
        let mut combined_ratings = Vec::with_capacity(num_row_ratings);
        for index in 0 .. row_ratings.len() {
            for row_rating in row_ratings[index] {
                combined_ratings.push(IndexedRowRating { index, row_rating });
            }
        }

        combined_ratings.sort_unstable_by(|a, b| {
            // Intentionally revert the order, so that high ratings appear first
            b.row_rating.rating.partial_cmp(&a.row_rating.rating).expect("NaN is impossible")
        });
        todo!()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct RowInfo {
    min_y: usize,
    height: usize,
    bound_x: usize
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct RowsInfo {
    rows: Vec<RowInfo>,
    atlas_width: usize,
}

impl RowsInfo {
    fn new(atlas_width: usize) -> Self {
        Self {
            rows: Vec::new(), atlas_width
        }
    }

    fn rank_placement_rows(&self, texture_width: usize, texture_height: usize) -> Vec<RowRating> {
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
