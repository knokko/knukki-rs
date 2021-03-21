/// Represents a group of texture atlases of limited size that work together to give the illusion of
/// being a single texture atlas with much bigger size. Not all textures will be in GPU memory at
/// any time, but this struct will make sure they are when they are needed there.
///
/// This struct methods to add textures to the group and methods to create models that refer to
/// such textures.
pub struct TextureAtlasGroup {
    max_num_atlases: usize,

    atlas_width: usize,
    atlas_height: usize,

    // TODO The internal structures
}

impl TextureAtlasGroup {
    pub fn new(max_num_atlases: usize, atlas_width: usize, atlas_height: usize) -> Self {
        Self {
            max_num_atlases,
            atlas_width,
            atlas_height
        }
    }
}