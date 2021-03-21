mod group;
mod id;
mod position;

pub use group::*;
pub use id::*;
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

    next_id: u64,
    placements: Vec<Rc<PlacedTexture>>,
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
    id: TextureID,

    position: Cell<Option<TextureAtlasPosition>>,

    // TODO Manage the priority somehow (for instance, increment each time it is used, and
    // periodically divide all priorities by 2)
    priority: Cell<u32>,
}

impl PlacedTexture {
    pub fn get_texture_id(&self) -> TextureID {
        self.id
    }

    pub fn is_valid(&self) -> bool {
        self.position.is_some()
    }

    pub fn invalidate(&self) {
        self.position.set(None);
    }

    pub fn get_position(&self) -> Option<TextureAtlasPosition> {
        self.position.get()
    }
}

impl TextureAtlas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            // We use a very weird background color (pink) because it should never be shown and it
            // will speed up debugging if it is shown for some reason
            big_texture: Texture::new(width, height, Color::rgb(200, 0, 100)),

            next_id: 0,
            placements: Vec::new(),
        }
    }

    pub fn add_textures(&mut self, textures: Vec<TextureID>, test: bool) -> TexturePlaceResult {
        // TODO Implement this!
    }
}