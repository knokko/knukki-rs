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
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct GroupTexturePlacement {
    cpu_atlas_index: u16,
    gpu_atlas_slot: u8,
    position: TextureAtlasPosition,
    still_valid: Rc<Cell<bool>>,
}

impl GroupTexturePlacement {
    #[cfg(test)]
    pub(crate) fn new(
        cpu_atlas_index: u16, gpu_atlas_slot: u8,
        position: TextureAtlasPosition, still_valid: Rc<Cell<bool>>
    ) -> Self {
        Self { cpu_atlas_index, gpu_atlas_slot, position, still_valid }
    }

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

struct AtlasEntry<GpuTexture> {
    atlas: TextureAtlas,
    gpu_texture: Option<(GpuTexture, u64)>,
}

/// Represents a group of texture atlases of limited size that work together to give the illusion of
/// being a single texture atlas with much bigger size. Not all textures will be in GPU memory at
/// any time, but this struct will make sure they are when they are needed there.
///
/// This has struct methods to add textures to the group and methods to create models that refer to
/// such textures.
pub struct TextureAtlasGroup<GpuTexture> {
    max_num_cpu_atlases: u16,
    max_num_gpu_atlases: u16,
    min_gpu_atlas_slot: u8,
    max_gpu_atlas_slot: u8,

    atlas_width: u32,
    atlas_height: u32,

    textures: HashMap<GroupTextureID, TextureEntry>,
    atlases: Vec<AtlasEntry<GpuTexture>>,

    next_texture_id: u64,

    // This variable is used to keep track of which gpu atlas texture are recently used
    current_time: u64,
}

impl<GpuTexture> TextureAtlasGroup<GpuTexture> {
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
            atlases: Vec::new(),

            next_texture_id: 0,
            current_time: 0
        }
    }

    /// Adds the given texture to this group and returns its `GroupTextureID`. Note that this
    /// method only stores the texture; it doesn't put it on any atlas yet. The returned id is
    /// needed for the `place_textures` method.
    pub fn add_texture(&mut self, texture: Texture) -> Result<GroupTextureID, TextureTooBigForAtlas> {

        if texture.get_width() > self.atlas_width || texture.get_height() > self.atlas_height {
            return Err(TextureTooBigForAtlas {
                texture_width: texture.get_width(),
                texture_height: texture.get_height(),
                atlas_width: self.atlas_width,
                atlas_height: self.atlas_height,
            });
        }

        let id = GroupTextureID { internal: self.next_texture_id };
        self.next_texture_id += 1;

        self.textures.insert(id, TextureEntry { texture, placements: Vec::new() });
        Ok(id)
    }

    pub fn remove_texture(&mut self, id: GroupTextureID) -> Result<(), ()> {
        todo!() // Also mark textures as removed, to improve debugging
    }

    pub fn get_texture(&self, id: GroupTextureID) -> &Texture {
        &self.textures[&id].texture
    }

    pub fn get_gpu_texture<GpuError, F: FnOnce(&Texture) -> Result<GpuTexture, GpuError>>(
        &mut self, atlas_index: u16, load_texture: F
    ) -> Result<&GpuTexture, GpuError> {
        self.current_time += 1;

        let is_ready = self.atlases[atlas_index as usize].gpu_texture.is_some();
        if !is_ready {

            let mut num_gpu_atlases = 1; // We also count the atlas that is about to be sent to gpu
            let mut least_recently_used_time = None;
            let mut least_recently_used_index = None;

            for current_index in 0 .. self.atlases.len() {
                let atlas_entry = &self.atlases[current_index];
                if let Some(gpu_entry) = &atlas_entry.gpu_texture {
                    num_gpu_atlases += 1;
                    if least_recently_used_time.is_none() || gpu_entry.1 < least_recently_used_time.unwrap() {
                        least_recently_used_time = Some(gpu_entry.1);
                        least_recently_used_index = Some(current_index);
                    }
                }
            }

            // Remove 1 gpu texture, if needed
            if num_gpu_atlases > self.max_num_gpu_atlases {
                self.atlases[least_recently_used_index.expect(
                    "There were too many gpu atlas textures, so there must be at least 1"
                )].gpu_texture = None;
            }

            self.atlases[atlas_index as usize].gpu_texture = Some((
                load_texture(self.atlases[atlas_index as usize].atlas.get_texture())?,
                self.current_time
            ));
        }

        Ok(&self.atlases[atlas_index as usize].gpu_texture.as_ref().unwrap().0)
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
            let test_place_result = self.atlases[atlas_index].atlas.add_textures(&textures_to_try, true);
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

                        // If made_progress were false, at least 1 texture would have to be too big
                        // for the texture atlas. But, adding such a texture should not be possible.
                        assert!(made_progress);

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
        let mut placements = HashMap::new();

        for dest_atlas_index in dest_atlas_indices {

            // We need to invalidate the gpu textures of all atlases we modify
            self.atlases[*dest_atlas_index].gpu_texture = None;

            let own_textures = &self.textures;

            let remaining_texture_ids: Vec<_> = texture_set.iter().filter(
                |texture_id| !placements.contains_key(*texture_id)
            ).collect();
            let remaining_textures: Vec<_> = remaining_texture_ids.iter().map(
                |texture_id| &own_textures[texture_id].texture
            ).collect();

            let place_result = self.atlases[*dest_atlas_index].atlas.add_textures(&remaining_textures, false);
            for index in 0 .. place_result.placements.len() {
                if let Some(placed_position) = place_result.placements[index].get_position() {

                    let gpu_atlas_slot = self.gpu_atlas_slot_for(*dest_atlas_index as u16);
                    placements.insert(*remaining_texture_ids[index], GroupTexturePlacement {
                        cpu_atlas_index: *dest_atlas_index as u16,
                        gpu_atlas_slot,
                        position: placed_position,
                        still_valid: Rc::new(Cell::new(true))
                    });
                }
            }
        }

        // If the input parameters were correct, all textures in the set should have been placed
        assert_eq!(placements.len(), texture_set.len());

        placements
    }

    fn place_textures_in_new_atlases(
        &mut self, texture_set: &HashSet<GroupTextureID>
    ) -> HashMap<GroupTextureID, GroupTexturePlacement> {

        let mut placements = HashMap::new();
        while placements.len() < texture_set.len() {

            let mut next_atlas = TextureAtlas::new(self.atlas_width, self.atlas_height);
            let remaining_texture_ids: Vec<_> = texture_set.iter().filter(
                |id| !placements.contains_key(*id)
            ).collect();
            let remaining_textures: Vec<_> = remaining_texture_ids.iter().map(
                |id| &self.textures[id].texture
            ).collect();

            let place_result = next_atlas.add_textures(&remaining_textures, false);
            let mut made_progress = false;
            for index in 0 .. place_result.placements.len() {
                if let Some(position) = place_result.placements[index].get_position() {
                    made_progress = true;

                    // This atlas will be added to the list of atlases, so its index will be the
                    // current length
                    let cpu_atlas_index = self.atlases.len() as u16;
                    let still_valid = Rc::new(Cell::new(true));

                    placements.insert(*remaining_texture_ids[index], GroupTexturePlacement {

                        cpu_atlas_index,
                        gpu_atlas_slot: self.gpu_atlas_slot_for(cpu_atlas_index),

                        position,
                        still_valid
                    });
                }
            }

            // If made_progress were false, at least 1 texture would have to be too big
            // for the texture atlas. But, adding such a texture should not be possible.
            assert!(made_progress);

            self.atlases.push(AtlasEntry {
                atlas: next_atlas,
                // Assigning GPU textures will be postponed until drawing
                gpu_texture: None
            });
        }

        placements
    }

    fn gpu_atlas_slot_for(&self, cpu_atlas_index: u16) -> u8 {
        let num_gpu_atlas_slots = 1 + self.max_gpu_atlas_slot - self.min_gpu_atlas_slot;

        // This is a very simple and powerful trick to map texture atlases evenly over gpu slots
        let gpu_atlas_slot_offset = cpu_atlas_index % (num_gpu_atlas_slots as u16);
        self.min_gpu_atlas_slot + gpu_atlas_slot_offset as u8
    }

    pub fn place_textures(&mut self, textures: &[GroupTextureID]) -> Vec<GroupTexturePlacement> {

        let mut texture_set = HashSet::with_capacity(textures.len());
        for texture_id in textures {
            texture_set.insert(*texture_id);
        }

        let existing_ratings = self.rate_texture_atlases(&texture_set);

        let maybe_dest_atlases = self.choose_texture_atlases(&texture_set, &existing_ratings);

        // Filter the textures that are already on at least 1 atlas
        let mut existing_placement_map = HashMap::new();
        if let Some(dest_atlases) = &maybe_dest_atlases {
            texture_set.retain(|texture_id| {
                for placement in &self.textures[texture_id].placements {
                    if dest_atlases.contains(&(placement.cpu_atlas_index as usize)) {
                        existing_placement_map.insert(*texture_id, placement.clone());
                        return false;
                    }
                }

                true
            });
        }

        let mut placement_map = match maybe_dest_atlases {
            Some(dest_atlases) => self.place_textures_at(&texture_set, &dest_atlases),
            None => self.place_textures_in_new_atlases(&texture_set)
        };

        // Update the textures map of this group
        for (texture_id, placement) in &placement_map {
            self.textures.get_mut(texture_id).unwrap().placements.push(placement.clone());
        }

        // Also add the existing entries to the result
        placement_map.extend(existing_placement_map.into_iter());

        textures.iter().map(|texture_id| placement_map[texture_id].clone()).collect()
    }
}

// This is just a helper struct for determining which texture atlas(es) to use
#[derive(Eq, PartialEq, Debug)]
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

#[cfg(test)]
mod tests {

    use super::*;

    use std::cell::Cell;
    use std::collections::HashSet;
    use std::rc::Rc;

    type TextureAtlasGroup = super::TextureAtlasGroup<()>;

    #[test]
    fn test_rate_texture_atlases() {
        let atlas_width = 10;
        let atlas_height = 10;
        let mut group = TextureAtlasGroup::new(
            atlas_width, atlas_height, 3,
            1, 1, 1
        );

        let texture1 = Texture::new(5, 4, Color::rgb(0, 0, 0));
        let texture2 = Texture::new(5, 4, Color::rgb(0, 0, 0));
        let texture3 = Texture::new(10, 6, Color::rgb(0, 0, 0));
        let texture4 = Texture::new(3, 3, Color::rgb(0, 0, 0));

        let id1 = group.add_texture(texture1).unwrap();
        let id2 = group.add_texture(texture2.clone()).unwrap();
        let id3 = group.add_texture(texture3).unwrap();
        let id4 = group.add_texture(texture4.clone()).unwrap();

        // Preparation code: this abuses access to the private members of TextureAtlasGroup to forge
        // a clear test case.
        group.atlases.push(AtlasEntry {
            atlas: TextureAtlas::new(atlas_width, atlas_height),
            gpu_texture: None
        });

        // Preparation: put texture2 on atlas 2
        let mut atlas2 = TextureAtlas::new(atlas_width, atlas_height);
        let position2 = TextureAtlasPosition {
            min_x: 0,
            min_y: 0,
            width: 5,
            height: 4
        };
        let place2 = atlas2.add_textures(&[&texture2], false);
        assert_eq!(Some(position2), place2.placements[0].get_position());
        group.atlases.push(AtlasEntry {
            atlas: atlas2,
            gpu_texture: None
        });
        let gpu_slot_1 = group.gpu_atlas_slot_for(1);
        group.textures.get_mut(&id2).unwrap().placements.push(GroupTexturePlacement {
            cpu_atlas_index: 1,
            gpu_atlas_slot: gpu_slot_1,
            position: position2,
            still_valid: Rc::new(Cell::new(true))
        });

        // Preparation: put texture 4 on atlas 3
        let mut atlas3 = TextureAtlas::new(atlas_width, atlas_height);
        let position3 = TextureAtlasPosition {
            min_x: 0,
            min_y: 0,
            width: 3,
            height: 3
        };
        let place3 = atlas3.add_textures(&[&texture4], false);
        assert_eq!(Some(position3), place3.placements[0].get_position());
        group.atlases.push(AtlasEntry {
            atlas: atlas3,
            gpu_texture: None
        });
        let gpu_atlas_slot2 = group.gpu_atlas_slot_for(2);
        group.textures.get_mut(&id4).unwrap().placements.push(GroupTexturePlacement {
            cpu_atlas_index: 2,
            gpu_atlas_slot: gpu_atlas_slot2,
            position: position3,
            still_valid: Rc::new(Cell::new(true))
        });

        // Now onto the actual test
        let mut texture_set = HashSet::new();
        texture_set.insert(id1);
        texture_set.insert(id2);
        texture_set.insert(id3);

        let ratings = group.rate_texture_atlases(&texture_set);

        // Atlas 2 should be the best candidate because it fits and contains 1 texture already
        assert!(ratings[0].fits);
        assert_eq!(1, ratings[0].atlas_index);
        assert_eq!(2, ratings[0].num_missing_textures);

        // Atlas 1 should be second because everything fits, but it doesn't contain any yet
        assert!(ratings[1].fits);
        assert_eq!(0, ratings[1].atlas_index);
        assert_eq!(3, ratings[1].num_missing_textures);

        // Atlas 3 should be last because not everything fits and it doesn't contain any of the
        // textures yet
        assert!(!ratings[2].fits);
        assert_eq!(2, ratings[2].atlas_index);
        assert_eq!(3, ratings[2].num_missing_textures);
    }

    #[test]
    fn test_choose_texture_atlases_empty() {
        let atlas_width = 5;
        let atlas_height = 9;
        let mut group = TextureAtlasGroup::new(
            atlas_width, atlas_height, 5, 2, 1, 1
        );

        let texture1 = Texture::new(3, 2, Color::rgb(0, 0, 0));
        let texture2 = Texture::new(3, 2, Color::rgb(0, 0, 0));

        let id1 = group.add_texture(texture1).unwrap();
        let id2 = group.add_texture(texture2).unwrap();
        let mut texture_set = HashSet::new();
        texture_set.insert(id1);
        texture_set.insert(id2);

        let ratings = group.rate_texture_atlases(&texture_set);
        assert!(ratings.is_empty());
        let test_result = group.choose_texture_atlases(&texture_set, &ratings);
        assert!(test_result.is_none());
    }

    #[test]
    fn test_choose_texture_atlases_fits_existing() {
        let atlas_width = 5;
        let atlas_height = 9;
        let mut group = TextureAtlasGroup::new(
            atlas_width, atlas_height, 5, 2, 1, 1
        );

        let texture1 = Texture::new(5, 7, Color::rgb(0, 0, 0));
        let texture2 = Texture::new(5, 7, Color::rgb(0, 0, 0));

        let id1 = group.add_texture(texture1).unwrap();
        let id2 = group.add_texture(texture2).unwrap();
        let mut texture_set = HashSet::new();
        texture_set.insert(id1);
        texture_set.insert(id2);

        // Let's prepare some fake data for the test
        group.atlases.push(AtlasEntry {
            atlas: TextureAtlas::new(atlas_width, atlas_height),
            gpu_texture: None
        });
        group.atlases.push(AtlasEntry {
            atlas: TextureAtlas::new(atlas_width, atlas_height),
            gpu_texture: None
        });

        let ratings1 = vec![
            ExistingAtlasRating {
                atlas_index: 0,
                num_missing_textures: 2,
                fits: true
            }, ExistingAtlasRating {
                atlas_index: 1,
                num_missing_textures: 2,
                fits: true
            }
        ];

        let test_result1 = group.choose_texture_atlases(&texture_set, &ratings1);
        assert_eq!(Some(vec![0]), test_result1);

        let ratings2 = vec![
            ExistingAtlasRating {
                atlas_index: 1,
                num_missing_textures: 2,
                fits: true
            }, ExistingAtlasRating {
                atlas_index: 0,
                num_missing_textures: 2,
                fits: false
            }
        ];

        let test_result2 = group.choose_texture_atlases(&texture_set, &ratings2);
        assert_eq!(Some(vec![1]), test_result2);
    }

    #[test]
    fn test_choose_texture_atlases_fits_no_existing() {
        let atlas_width = 5;
        let atlas_height = 9;
        let mut group = TextureAtlasGroup::new(
            atlas_width, atlas_height, 5, 2, 1, 1
        );

        let texture1 = Texture::new(5, 7, Color::rgb(0, 0, 0));
        let texture2 = Texture::new(5, 7, Color::rgb(0, 0, 0));

        let id1 = group.add_texture(texture1).unwrap();
        let id2 = group.add_texture(texture2).unwrap();
        let mut texture_set = HashSet::new();
        texture_set.insert(id1);
        texture_set.insert(id2);

        // Let's prepare some fake data for the test
        group.atlases.push(AtlasEntry {
            atlas: TextureAtlas::new(atlas_width, atlas_height),
            gpu_texture: None
        });

        let ratings = vec![ExistingAtlasRating {
            atlas_index: 0,
            num_missing_textures: 2,
            fits: false
        }];

        let test_result1 = group.choose_texture_atlases(&texture_set, &ratings);
        assert!(test_result1.is_none());
    }

    #[test]
    fn test_place_textures_in_new_atlases() {
        let atlas_width = 10;
        let atlas_height = 7;

        let mut group = TextureAtlasGroup::new(
            atlas_width, atlas_height, 5, 1, 1, 1
        );

        let color1 = Color::rgb(255, 0, 0);
        let color2 = Color::rgb(0, 255, 0);
        let color3 = Color::rgb(0, 0, 255);
        let texture1 = Texture::new(6, 6, color1);
        let texture2 = Texture::new(3, 3, color2);
        let texture3 = Texture::new(5, 5, color3);
        let id1 = group.add_texture(texture1).unwrap();
        let id2 = group.add_texture(texture2).unwrap();
        let id3 = group.add_texture(texture3).unwrap();

        let mut texture_set1 = HashSet::new();
        texture_set1.insert(id1);
        texture_set1.insert(id2);

        let mut texture_set2 = texture_set1.clone();
        texture_set2.insert(id3);

        let place_result1 = group.place_textures_in_new_atlases(&texture_set1);
        assert_eq!(2, place_result1.len());
        assert_eq!(GroupTexturePlacement {
            cpu_atlas_index: 0,
            gpu_atlas_slot: group.gpu_atlas_slot_for(0),
            position: TextureAtlasPosition {
                min_x: 0,
                min_y: 0,
                width: 6,
                height: 6
            },
            still_valid: Rc::new(Cell::new(true))
        }, place_result1[&id1]);
        assert_eq!(GroupTexturePlacement {
            cpu_atlas_index: 0,
            gpu_atlas_slot: group.gpu_atlas_slot_for(0),
            position: TextureAtlasPosition {
                min_x: 6,
                min_y: 0,
                width: 3,
                height: 3
            },
            still_valid: Rc::new(Cell::new(true))
        }, place_result1[&id2]);
        assert_eq!(color1, group.atlases[0].atlas.get_texture()[0][0]);
        assert_eq!(color2, group.atlases[0].atlas.get_texture()[6][0]);

        let place_result2 = group.place_textures_in_new_atlases(&texture_set2);
        assert_eq!(3, place_result2.len());
        assert_eq!(GroupTexturePlacement {
            cpu_atlas_index: 1,
            gpu_atlas_slot: group.gpu_atlas_slot_for(1),
            position: TextureAtlasPosition {
                min_x: 0,
                min_y: 0,
                width: 6,
                height: 6
            },
            still_valid: Rc::new(Cell::new(true))
        }, place_result2[&id1]);
        assert_eq!(GroupTexturePlacement {
            cpu_atlas_index: 1,
            gpu_atlas_slot: group.gpu_atlas_slot_for(1),
            position: TextureAtlasPosition {
                min_x: 6,
                min_y: 0,
                width: 3,
                height: 3
            },
            still_valid: Rc::new(Cell::new(true))
        }, place_result2[&id2]);
        assert_eq!(GroupTexturePlacement {
            cpu_atlas_index: 2,
            gpu_atlas_slot: group.gpu_atlas_slot_for(2),
            position: TextureAtlasPosition {
                min_x: 0,
                min_y: 0,
                width: 5,
                height: 5
            },
            still_valid: Rc::new(Cell::new(true))
        }, place_result2[&id3]);

        assert_eq!(color1, group.atlases[0].atlas.get_texture()[0][0]);
        assert_eq!(color2, group.atlases[0].atlas.get_texture()[6][0]);
        assert_eq!(color1, group.atlases[1].atlas.get_texture()[0][0]);
        assert_eq!(color2, group.atlases[1].atlas.get_texture()[6][0]);
        assert_eq!(color3, group.atlases[2].atlas.get_texture()[0][0]);
    }

    #[test]
    fn test_place_textures_at() {
        let atlas_width = 5;
        let atlas_height = 8;

        let mut group = TextureAtlasGroup::new(
            atlas_width, atlas_height, 5, 1, 1, 1
        );

        let color1 = Color::rgb(255, 0, 0);
        let color2 = Color::rgb(0, 255, 0);
        let color3 = Color::rgb(0, 0, 255);

        let texture1 = Texture::new(5, 4, color1);
        let texture2 = Texture::new(3, 2, color2);
        let texture3 = Texture::new(2, 3, color3);

        let id1 = group.add_texture(texture1).unwrap();
        let id2 = group.add_texture(texture2).unwrap();
        let id3 = group.add_texture(texture3).unwrap();

        // Create empty atlases 1, 2, and 3
        for _ in 0 .. 3 {
            group.atlases.push(AtlasEntry {
                atlas: TextureAtlas::new(atlas_width, atlas_height),
                gpu_texture: None
            });
        }

        let mut texture_set1 = HashSet::new();
        texture_set1.insert(id1);

        // This should place texture1 at (0, 0) in atlas 3
        let test_result1 = group.place_textures_at(&texture_set1, &vec![2]);
        assert_eq!(1, test_result1.len());
        assert_eq!(GroupTexturePlacement {
            cpu_atlas_index: 2,
            gpu_atlas_slot: group.gpu_atlas_slot_for(2),
            position: TextureAtlasPosition {
                min_x: 0,
                min_y: 0,
                width: 5,
                height: 4
            },
            still_valid: Rc::new(Cell::new(true))
        }, test_result1[&id1]);
        assert_eq!(color1, group.atlases[2].atlas.get_texture()[0][0]);

        let mut texture_set2 = texture_set1.clone();
        texture_set2.insert(id2);
        texture_set2.insert(id3);

        /*
         * This should place texture 1 in the remaining space of atlas 3 and it should place texture
         * 2 and texture 3 at the first row in atlas 1.
         */
        let test_result2 = group.place_textures_at(
            &texture_set2, &vec![2, 0]
        );
        assert_eq!(3, test_result2.len());
        assert_eq!(GroupTexturePlacement {
            cpu_atlas_index: 2,
            gpu_atlas_slot: group.gpu_atlas_slot_for(2),
            position: TextureAtlasPosition {
                min_x: 0,
                min_y: 4,
                width: 5,
                height: 4
            },
            still_valid: Rc::new(Cell::new(true))
        }, test_result2[&id1]);
        assert_eq!(color1, group.atlases[2].atlas.get_texture()[0][4]);
        assert_eq!(GroupTexturePlacement {
            cpu_atlas_index: 0,
            gpu_atlas_slot: group.gpu_atlas_slot_for(0),
            position: TextureAtlasPosition {
                min_x: 0,
                min_y: 0,
                width: 2,
                height: 3
            },
            still_valid: Rc::new(Cell::new(true))
        }, test_result2[&id3]);
        assert_eq!(color3, group.atlases[0].atlas.get_texture()[0][0]);
        assert_eq!(GroupTexturePlacement {
            cpu_atlas_index: 0,
            gpu_atlas_slot: group.gpu_atlas_slot_for(0),
            position: TextureAtlasPosition {
                min_x: 2,
                min_y: 0,
                width: 3,
                height: 2
            },
            still_valid: Rc::new(Cell::new(true))
        }, test_result2[&id2]);
        assert_eq!(color2, group.atlases[0].atlas.get_texture()[2][0]);
    }

    #[test]
    fn test_place_textures() {
        let atlas_width = 50;
        let atlas_height = 35;

        let mut group = TextureAtlasGroup::new(
            atlas_width, atlas_height, 7, 6, 3, 4
        );

        let color1 = Color::rgb(100, 100, 0);
        let color2 = Color::rgb(100, 0, 200);
        let color3 = Color::rgb(0, 100, 200);
        let color4 = Color::rgb(200, 0, 200);
        let color5 = Color::rgb(100, 200, 150);
        let color6 = Color::rgb(100, 100, 100);
        let color7 = Color::rgb(200, 200, 200);

        let texture1 = Texture::new(20, 15, color1);
        let texture2 = Texture::new(15, 17, color2);
        let texture3 = Texture::new(13, 12, color3);
        let texture4 = Texture::new(16, 18, color4);
        let texture5 = Texture::new(10, 16, color5);
        let texture6 = Texture::new(15, 5, color6);
        let texture7 = Texture::new(17, 15, color7);

        let id1 = group.add_texture(texture1).unwrap();
        let id2 = group.add_texture(texture2).unwrap();
        let id3 = group.add_texture(texture3).unwrap();
        let id4 = group.add_texture(texture4).unwrap();
        let id5 = group.add_texture(texture5).unwrap();
        let id6 = group.add_texture(texture6).unwrap();
        let id7 = group.add_texture(texture7).unwrap();

        // The first 4 textures, plus some duplicates that should be ignored
        let texture_list_1 = [id1, id2, id1, id3, id4, id2];
        let test_result1 = group.place_textures(&texture_list_1);

        // First the global tests
        assert_eq!(texture_list_1.len(), test_result1.len());
        assert_eq!(1, group.atlases.len());
        assert!(group.gpu_atlas_slot_for(0) >= 3 && group.gpu_atlas_slot_for(0) <= 4);
        for index in 0 .. texture_list_1.len() {
            assert_eq!(0, test_result1[index].cpu_atlas_index);
            assert_eq!(group.gpu_atlas_slot_for(0), test_result1[index].gpu_atlas_slot);
            assert!(test_result1[index].is_still_valid());
            assert!(group.textures[&texture_list_1[index]].placements.contains(&test_result1[index]));

            for index2 in 0 .. texture_list_1.len() {
                if index == index2 {
                    assert_eq!(test_result1[index], test_result1[index2]);
                }
            }
        }

        // Now the specific tests
        assert_eq!(color1, group.atlases[0].atlas.get_texture()[0][18]);
        assert_eq!(TextureAtlasPosition {
            min_x: 0,
            min_y: 18,
            width: 20,
            height: 15
        }, test_result1[0].position);

        assert_eq!(color2, group.atlases[0].atlas.get_texture()[16][0]);
        assert_eq!(TextureAtlasPosition {
            min_x: 16,
            min_y: 0,
            width: 15,
            height: 17
        }, test_result1[1].position);

        assert_eq!(color3, group.atlases[0].atlas.get_texture()[20][18]);
        assert_eq!(TextureAtlasPosition {
            min_x: 20,
            min_y: 18,
            width: 13,
            height: 12
        }, test_result1[3].position);

        assert_eq!(color4, group.atlases[0].atlas.get_texture()[0][0]);
        assert_eq!(TextureAtlasPosition {
            min_x: 0,
            min_y: 0,
            width: 16,
            height: 18
        }, test_result1[4].position);

        // The next test is to place the first 5 textures. This should fit onto the first atlas
        // because the first 4 textures are on that atlas already.
        let texture_list_2 = [id4, id1, id5, id4, id2, id5, id3];
        let test_result2 = group.place_textures(&texture_list_2);

        // First the global tests
        assert_eq!(texture_list_2.len(), test_result2.len());
        assert_eq!(1, group.atlases.len());
        for index in 0 .. texture_list_2.len() {
            assert_eq!(0, test_result2[index].cpu_atlas_index);
            assert_eq!(group.gpu_atlas_slot_for(0), test_result2[index].gpu_atlas_slot);
            assert!(test_result2[index].is_still_valid());
            assert!(group.textures[&texture_list_2[index]].placements.contains(&test_result2[index]));

            for index2 in 0 .. texture_list_2.len() {
                if index == index2 {
                    assert_eq!(test_result2[index], test_result2[index2]);
                }
            }
        }

        // Also test that the placements for test 1 are still valid
        for placed in &test_result1 {
            assert!(placed.is_still_valid());
        }

        // Now the specific tests (note that the first 4 were mostly copied from test 1)
        assert_eq!(color1, group.atlases[0].atlas.get_texture()[0][18]);
        assert_eq!(TextureAtlasPosition {
            min_x: 0,
            min_y: 18,
            width: 20,
            height: 15
        }, test_result2[1].position);

        assert_eq!(color2, group.atlases[0].atlas.get_texture()[16][0]);
        assert_eq!(TextureAtlasPosition {
            min_x: 16,
            min_y: 0,
            width: 15,
            height: 17
        }, test_result2[4].position);

        assert_eq!(color3, group.atlases[0].atlas.get_texture()[20][18]);
        assert_eq!(TextureAtlasPosition {
            min_x: 20,
            min_y: 18,
            width: 13,
            height: 12
        }, test_result2[6].position);

        assert_eq!(color4, group.atlases[0].atlas.get_texture()[0][0]);
        assert_eq!(TextureAtlasPosition {
            min_x: 0,
            min_y: 0,
            width: 16,
            height: 18
        }, test_result2[0].position);

        assert_eq!(color5, group.atlases[0].atlas.get_texture()[31][0]);
        assert_eq!(TextureAtlasPosition {
            min_x: 31,
            min_y: 0,
            width: 10,
            height: 16
        }, test_result2[2].position);

        // This is the last texture that should fit on texture atlas 1
        let texture_list_3 = [id6, id6, id6];
        let test_result3 = group.place_textures(&texture_list_3);

        assert_eq!(3, test_result3.len());
        let position6 = TextureAtlasPosition {
            min_x: 33,
            min_y: 18,
            width: 15,
            height: 5
        };
        for index in 0 .. 3 {
            assert_eq!(0, test_result3[index].cpu_atlas_index);
            assert_eq!(group.gpu_atlas_slot_for(0), test_result3[index].gpu_atlas_slot);
            assert_eq!(position6, test_result3[index].position);
        }
        assert_eq!(1, group.textures[&id6].placements.len());
        assert_eq!(position6, group.textures[&id6].placements[0].position);
        assert_eq!(color6, group.atlases[0].atlas.get_texture()[33][18]);

        // The final test is to add 1 more texture, along with 2 existing textures. Due to the first
        // 6 textures, there is no more space on atlas 1, so a new atlas will have to be created.
        // The group should avoid splitting textures placed in the same call across multiple
        // atlases. To avoid this, it will have to copy the 2 existing textures to atlas 2 as well,
        // but without removing it from atlas 1.
        let texture_list_4 = [id2, id7, id4];
        let test_result4 = group.place_textures(&texture_list_4);
        assert_eq!(3, test_result4.len());
        assert_eq!(2, group.atlases.len());
        for placement in &test_result4 {
            assert_eq!(1, placement.cpu_atlas_index);
            assert_eq!(group.gpu_atlas_slot_for(1), placement.gpu_atlas_slot);
        }

        assert_eq!(color2, group.atlases[1].atlas.get_texture()[16][0]);
        assert_eq!(TextureAtlasPosition {
            min_x: 16,
            min_y: 0,
            width: 15,
            height: 17
        }, test_result4[0].position);

        assert_eq!(color7, group.atlases[1].atlas.get_texture()[31][0]);
        assert_eq!(TextureAtlasPosition {
            min_x: 31,
            min_y: 0,
            width: 17,
            height: 15
        }, test_result4[1].position);

        assert_eq!(color4, group.atlases[1].atlas.get_texture()[0][0]);
        assert_eq!(TextureAtlasPosition {
            min_x: 0,
            min_y: 0,
            width: 16,
            height: 18
        }, test_result4[2].position);

        // Time to check the final state
        for test_result in &[&test_result1, &test_result2, &test_result3, &test_result4] {
            for placement in *test_result {
                assert!(placement.is_still_valid());
            }
        }

        assert_eq!(color1, group.atlases[0].atlas.get_texture()[0][18]);
        assert_eq!(color2, group.atlases[0].atlas.get_texture()[16][0]);
        assert_eq!(color2, group.atlases[1].atlas.get_texture()[16][0]);
        assert_eq!(color3, group.atlases[0].atlas.get_texture()[20][18]);
        assert_eq!(color4, group.atlases[0].atlas.get_texture()[0][0]);
        assert_eq!(color4, group.atlases[1].atlas.get_texture()[0][0]);
        assert_eq!(color5, group.atlases[0].atlas.get_texture()[31][0]);
        assert_eq!(color6, group.atlases[0].atlas.get_texture()[33][18]);
        assert_eq!(color7, group.atlases[1].atlas.get_texture()[31][0]);

        assert_eq!(1, group.textures[&id1].placements.len());
        assert_eq!(2, group.textures[&id2].placements.len());
        assert_eq!(1, group.textures[&id3].placements.len());
        assert_eq!(2, group.textures[&id4].placements.len());
        assert_eq!(1, group.textures[&id5].placements.len());
        assert_eq!(1, group.textures[&id6].placements.len());
        assert_eq!(1, group.textures[&id7].placements.len());

        for texture_id in &[id1, id2, id3, id4, id5, id6, id7] {
            for placement in &group.textures[texture_id].placements {
                assert!(placement.is_still_valid());
            }
        }
    }

    #[test]
    fn test_unload_gpu_texture_after_edit() {

        let mut group = TextureAtlasGroup::new(
            10, 10, 2, 2, 2, 2
        );

        let test_color = Color::rgb(0, 0, 0);
        let texture1 = Texture::new(10, 5, test_color);
        let texture2 = Texture::new(10, 5, test_color);

        let id1 = group.add_texture(texture1).unwrap();
        let id2 = group.add_texture(texture2).unwrap();

        group.place_textures(&[id1]);

        group.get_gpu_texture::<(), _>(0, |_texture| Ok(())).unwrap();
        assert!(group.atlases[0].gpu_texture.is_some());

        group.place_textures(&[id2]);
        assert!(group.atlases[0].gpu_texture.is_none());
    }

    #[test]
    fn test_unload_gpu_texture_lru() {
        let test_color = Color::rgb(0, 0, 0);

        let texture1 = Texture::new(10, 10, test_color);
        let texture2 = Texture::new(10, 10, test_color);
        let texture3 = Texture::new(10, 10, test_color);

        let mut group = super::TextureAtlasGroup::new(
            10, 10, 10, 2, 1, 2
        );

        let id1 = group.add_texture(texture1).unwrap();
        let id2 = group.add_texture(texture2).unwrap();
        let id3 = group.add_texture(texture3).unwrap();

        group.place_textures(&[id1, id2, id3]);
        assert_eq!(3, group.atlases.len());

        group.get_gpu_texture::<(), _>(1, |_texture| Ok(1)).unwrap();
        assert!(group.atlases[0].gpu_texture.is_none());
        assert_eq!(1, group.atlases[1].gpu_texture.unwrap().0);
        assert!(group.atlases[2].gpu_texture.is_none());

        group.get_gpu_texture::<(), _>(0, |_texture| Ok(0)).unwrap();
        assert_eq!(0, group.atlases[0].gpu_texture.unwrap().0);
        assert_eq!(1, group.atlases[1].gpu_texture.unwrap().0);
        assert!(group.atlases[2].gpu_texture.is_none());

        // max_num_gpu_atlases is 2, so it will have to drop the oldest one (the second atlas)
        group.get_gpu_texture::<(), _>(2, |_texture| Ok(2)).unwrap();
        assert_eq!(0, group.atlases[0].gpu_texture.unwrap().0);
        assert!(group.atlases[1].gpu_texture.is_none());
        assert_eq!(2, group.atlases[2].gpu_texture.unwrap().0);

        // Now it should drop the first one
        group.get_gpu_texture::<(), _>(1, |_texture| Ok(3)).unwrap();
        assert!(group.atlases[0].gpu_texture.is_none());
        assert_eq!(3, group.atlases[1].gpu_texture.unwrap().0);
        assert_eq!(2, group.atlases[2].gpu_texture.unwrap().0);
    }
}
