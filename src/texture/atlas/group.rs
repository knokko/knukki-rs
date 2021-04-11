use crate::*;

use std::cell::Cell;
use std::cmp::{
    PartialOrd,
    Ord,
    Ordering,
};
use std::collections::{
    HashMap,
    HashSet,
};
use std::rc::Rc;

/// Represents the id/handle of a `Texture` within a `TextureAtlasGroup`. Instances of this struct
/// can be obtained by using the `add_texture` method of a `TextureAtlasGroup`.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct GroupTextureID {
    internal: u64,
}

/// Represents the placement of a `Texture` onto a `TextureAtlas` of a `TextureAtlasGroup`. See the
/// documentation of the methods of this struct for more information.
#[derive(Clone)]
pub struct GroupTexturePlacement {
    cpu_atlas_index: u16,
    gpu_atlas_slot: u8,
    position: TextureAtlasPosition,
    still_valid: Rc<Cell<bool>>,
}

impl GroupTexturePlacement {
    /// Gets the index/id of the texture atlas in a `TextureAtlasGroup` on which the corresponding
    /// texture is stored.
    pub fn get_cpu_atlas_index(&self) -> u16 {
        self.cpu_atlas_index
    }

    /// Gets the gpu slot in which the texture atlas identified by `get_cpu_atlas_index` should be
    /// put.
    pub fn get_gpu_atlas_slot(&self) -> u8 {
        self.gpu_atlas_slot
    }

    /// Gets the position of the corresponding texture within the texture atlas.
    pub fn get_position(&self) -> TextureAtlasPosition {
        self.position
    }

    /// Invalidates this placement. This should be used when the corresponding texture has been
    /// removed from its texture atlas (for instance to make place for another texture).
    pub fn invalidate(&self) {
        self.still_valid.set(false);
    }

    /// Checks if this placement is still valid. If so, the corresponding texture is still located
    /// at the given position in the given texture atlas. If not, the corresponding texture has
    /// been moved or removed. In that case, any model that relies on this placement should be
    /// recreated.
    pub fn is_still_valid(&self) -> bool {
        self.still_valid.get()
    }
}

struct TextureEntry {
    texture: Texture,
    placements: Vec<GroupTexturePlacement>,
}

/// Represents a group of texture atlases of limited size that work together to give the illusion of
/// being a single texture atlas with much bigger size. Not all textures will be in GPU memory at
/// any time, but this struct will make sure they are when they are needed there.
///
/// This has struct methods to add textures to the group and methods to create models that refer to
/// such textures.
pub struct TextureAtlasGroup {
    max_num_cpu_atlases: u16,
    max_num_gpu_atlases: u16,
    min_gpu_atlas_slot: u8,
    max_gpu_atlas_slot: u8,

    atlas_width: u32,
    atlas_height: u32,

    textures: HashMap<GroupTextureID, TextureEntry>,
    atlases: Vec<TextureAtlas>,

    next_texture_id: u64,
}

impl TextureAtlasGroup {
    /// Constructs a new `TextureAtlasGroup` with the given parameters:
    ///
    /// ### Atlas width
    /// The `atlas_width` is the width (in pixels) of all texture atlases that will be created by
    /// this group.
    ///
    /// ### Atlas height
    /// The `atlas_height` is the height (in pixels) of all texture atlases that will be created by
    /// this group.
    ///
    /// ### Maximum number of texture atlases in CPU memory
    /// `max_num_cpu_atlases` is the maximum number of texture atlases that the group will store in
    /// CPU memory. When that number of atlases is reached and all atlases are full, the group will
    /// remove textures from existing atlases to make space for new textures when needed.
    ///
    /// ### Maximum number of texture atlases in GPU memory
    /// `max_num_gpu_atlases` is the maximum number of texture atlases that the group will
    /// simultaneously keep in GPU memory. When this number is reached, the group will remove an
    /// existing texture atlas from GPU memory when it needs to make place for a new texture atlas
    /// in GPU memory.
    ///
    /// ### GPU atlas slots
    /// Texture atlas groups assume that the GPU has a fixed number of texture slots that can be
    /// referenced in shaders during draw calls, by using their numerical id. When texture atlas
    /// groups need to place related textures onto different atlases, they will try to assign
    /// different GPU slot numbers to the placements in different texture atlases (which makes it
    /// possible to draw the related textures during the same draw call). `min_gpu_atlas_slot` is
    /// the smallest slot number that the group will use and `max_gpu_atlas_slot` is the largest
    /// slot number that the group will use. If these values are the same, the atlas texture group
    /// will only use 1 GPU texture slot. That is allowed, but giving it more slots can improve
    /// performance if there are a lot of textures (to see what works best, just try some values).
    ///
    /// ### Panics
    /// This will panic if any of the following conditions hold:
    ///
    /// (*) `atlas_width == 0`
    ///
    /// (*) `atlas_height == 0`
    ///
    /// (*) `max_num_cpu_atlases < max_num_gpu_atlases` (Currently, we always maintain a copy of
    /// every GPU texture atlas on the CPU, so allowing more GPU atlases than CPU atlases makes no
    /// sense.)
    ///
    /// (*) `max_num_gpu_atlases < (1 + max_gpu_atlas_slot - min_gpu_atlas_slot)` (If this were
    /// true, some slots would be completely unusable)
    pub fn new(
        atlas_width: u32, atlas_height: u32,
        max_num_cpu_atlases: u16, max_num_gpu_atlases: u16,
        min_gpu_atlas_slot: u8, max_gpu_atlas_slot: u8,
    ) -> Self {

        // Cheap sanity checks
        assert_ne!(0, atlas_width);
        assert_ne!(0, atlas_height);
        assert_ne!(0, max_num_cpu_atlases);
        assert!(max_num_cpu_atlases >= max_num_gpu_atlases);
        assert!(max_num_gpu_atlases >= (1 + max_gpu_atlas_slot - min_gpu_atlas_slot) as u16);
        assert!(max_gpu_atlas_slot >= min_gpu_atlas_slot);

        Self {
            atlas_width,
            atlas_height,

            max_num_cpu_atlases,
            max_num_gpu_atlases,
            min_gpu_atlas_slot,
            max_gpu_atlas_slot,

            textures: HashMap::new(),
            atlases: Vec::with_capacity(max_num_cpu_atlases as usize),

            next_texture_id: 0,
        }
    }

    /// Adds the given texture to this group and returns its `GroupTextureID`. Note that this
    /// method only stores the texture; it doesn't put it on any atlas yet. The returned id is
    /// needed for the `place_textures` method.
    pub fn add_texture(&mut self, texture: Texture) -> GroupTextureID {
        let id = GroupTextureID { internal: self.next_texture_id };
        self.next_texture_id += 1;

        self.textures.insert(id, TextureEntry { texture, placements: Vec::new() });
        id
    }

    fn rate_texture_atlases(&mut self, texture_set: &HashSet<GroupTextureID>) -> Vec<ExistingAtlasRating> {
        let mut existing_ratings = Vec::with_capacity(self.atlases.len());
        for atlas_index in 0 .. self.atlases.len() {

            // To rate the atlas, determine how many textures are still missing, and if they fit
            let mut remaining_textures = Vec::with_capacity(texture_set.len());
            for texture_id in texture_set {

                let texture_entry = &self.textures[texture_id];
                if !texture_entry.placements.iter().any(|placement|
                    placement.cpu_atlas_index as usize == atlas_index && placement.is_still_valid()
                ) {
                    remaining_textures.push(*texture_id);
                }
            }

            let my_textures = &self.textures;
            let textures_to_try: Vec<_> = remaining_textures.iter().map(
                |texture_id| &my_textures[texture_id].texture
            ).collect();
            let test_place_result = self.atlases[atlas_index].add_textures(&textures_to_try, true);
            let test_placed_all = !test_place_result.placements.iter().any(|test_placement| {
                !test_placement.is_valid()
            });

            let rating = ExistingAtlasRating {
                atlas_index: atlas_index as u16,
                num_missing_textures: remaining_textures.len() as u32,
                fits: test_place_result.num_replaced_textures == 0 && test_placed_all
            };

            existing_ratings.push(rating);
        }

        existing_ratings.sort_unstable();
        existing_ratings.reverse();

        existing_ratings
    }

    fn choose_texture_atlases(
        &self, texture_set: &HashSet<GroupTextureID>, existing_ratings: &Vec<ExistingAtlasRating>
    ) -> Option<Vec<usize>> {

        match existing_ratings.is_empty() {
            true => None,
            false => {
                if existing_ratings.first().unwrap().fits {
                    // If all textures can fit on an existing atlas, use that atlas
                    Some(vec![existing_ratings.first().unwrap().atlas_index as usize])
                } else {
                    // Try to place all textures on new texture atlases, and see how many we would
                    // need...
                    let mut num_needed_atlases = 0;
                    let mut texture_ids: Vec<_> = texture_set.iter().map(
                        |id| Some(*id)
                    ).collect();

                    let mut dummy_atlas = TextureAtlas::new(self.atlas_width, self.atlas_height);
                    loop {

                        let remaining_textures: Vec<_> = texture_ids.iter().filter_map(
                            |maybe_id| maybe_id.map(|id| &self.textures[&id].texture)
                        ).collect();

                        if remaining_textures.is_empty() {
                            break;
                        }

                        let test_result = dummy_atlas.add_textures(&remaining_textures, true);
                        let mut made_progress = false;
                        for index in 0 .. test_result.placements.len() {
                            if test_result.placements[index].is_valid() {
                                texture_ids[index] = None;
                                made_progress = true;
                            }
                        }

                        if !made_progress {
                            todo!() // We need some kind of error handling for this edge case
                        }
                        num_needed_atlases += 1;
                    }
                    if self.atlases.len() + num_needed_atlases <= self.max_num_cpu_atlases as usize {
                        None
                    } else {
                        // We will have to remove textures from an existing atlas...
                        todo!()
                    }
                }
            }
        }
    }

    fn place_textures_at(
        &mut self, texture_set: &HashSet<GroupTextureID>, dest_atlas_indices: &Vec<usize>
    ) -> HashMap<GroupTextureID, GroupTexturePlacement> {
        todo!()
    }

    fn place_textures_in_new_atlases(
        &mut self, texture_set: &HashSet<GroupTextureID>
    ) -> HashMap<GroupTextureID, GroupTexturePlacement> {
        todo!()
    }

    pub fn place_textures(&mut self, textures: &[GroupTextureID]) -> Vec<GroupTexturePlacement> {

        let mut texture_set = HashSet::with_capacity(textures.len());
        for texture_id in textures {
            texture_set.insert(*texture_id);
        }

        let existing_ratings = self.rate_texture_atlases(&texture_set);

        let maybe_dest_atlases = self.choose_texture_atlases(&texture_set, &existing_ratings);

        let placement_map = match maybe_dest_atlases {
            Some(dest_atlases) => self.place_textures_at(&texture_set, &dest_atlases),
            None => self.place_textures_in_new_atlases(&texture_set)
        };

        textures.iter().map(|texture_id| placement_map[texture_id].clone()).collect()
    }
}

// This is just a helper struct for determining which texture atlas(es) to use
#[derive(Eq, PartialEq)]
struct ExistingAtlasRating {
    atlas_index: u16,
    num_missing_textures: u32,
    fits: bool,
}

impl PartialOrd for ExistingAtlasRating {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExistingAtlasRating {
    fn cmp(&self, other: &Self) -> Ordering {
        // Most important is whether the textures fit
        if self.fits && !other.fits {
            return Ordering::Greater;
        }
        if !self.fits && other.fits {
            return Ordering::Less;
        }

        // If the fit test results in a tie, the number of missing textures is checked
        if self.num_missing_textures < other.num_missing_textures {
            return Ordering::Greater;
        }
        if self.num_missing_textures > other.num_missing_textures {
            return Ordering::Less;
        }

        // If the number of missing textures also result in a tie, the result doesn't really matter
        return self.atlas_index.cmp(&other.atlas_index)
    }
}