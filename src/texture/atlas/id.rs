/// Represents a unique identifier of a texture within a texture atlas (group). The only constructor
/// function of this struct has a limited visibility, so instances of this struct can only be
/// created internally by this crate.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct TextureID {
    value: u64
}

impl TextureID {
    pub(super) fn new(value: u64) -> Self {
        Self { value }
    }

    pub fn get_value(&self) -> u64 {
        self.value
    }
}