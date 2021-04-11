use std::fmt::{
    Display,
    Formatter,
    Result,
};

/// This error is used to indicate that an attempt was made to place a texture on a
/// TextureAtlas(Group), but the width of the texture was greater than the width of the atlas(es)
/// or the height of the texture was greater than the height of the atlas(es).
///
/// In such cases, it would be impossible to place the texture on an atlas, even if all existing
/// textures would be removed.
#[derive(Copy, Clone, Debug, Error)]
pub struct TextureTooBigForAtlas {
    pub texture_width: u32,
    pub texture_height: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
}

impl Display for TextureTooBigForAtlas {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter,
               "Texture(width={}, height={}) can't fit on an atlas of size (width={}, height={})",
                self.texture_width, self.texture_height, self.atlas_width, self.atlas_height
        )
    }
}
