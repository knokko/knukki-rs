/// Describe the region of the viewport where a `Component` is allowed to render
/// itself. This is normally the *domain* of the component.
///
/// The render region of the component will be given as parameter to its `render`
/// method. The parent component or provider will ensure that the viewport is set
/// to that render region before calling the `render` method.
///
/// ### Methods
/// The component is free to query the properties of the render region (like its
/// width, minimum y coordinate, aspect ratio...). Especially the aspect ratio
/// should be useful for many components, because that is the only way they can
/// prevent distortion. Most components should have no need for the other
/// properties.
///
/// This struct also has a `child_region` method, which can be useful for menu
/// components to create regions for its child components.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct RenderRegion {
    min_x: u32,
    min_y: u32,
    width: u32,
    height: u32,
}

impl RenderRegion {
    /// Constructs a new `RenderRegion` with the given minimum x-coordinate,
    /// minimum y-coordinate, width, and height.
    ///
    /// ## Panics
    /// This function will panic if `width == 0` or `height == 0`
    pub fn with_size(min_x: u32, min_y: u32, width: u32, height: u32) -> Self {
        if width == 0 || height == 0 {
            panic!("width is {} and height is {}", width, height);
        }
        Self {
            min_x,
            min_y,
            width,
            height,
        }
    }

    /// Constructs a new `RenderRegion` with the given minimum x-coordinate,
    /// minimum y-coordinate, bound x-coordinate, and bound y-coordinate.
    ///
    /// The bound x-coordinate is the x-coordinate that comes right after the
    /// maximum (right) x-coordinate and the bound y-coordinate is the y-coordinate
    /// that comes right after the maximum (bottom) y-coordinate.
    ///
    /// ## Panic
    /// This function will panic if `bound_x <= min_x` or `bound_y <= min_y`
    pub fn between(min_x: u32, min_y: u32, bound_x: u32, bound_y: u32) -> Self {
        if bound_x <= min_x {
            panic!("Bound x is {}, but min x is {}", bound_x, min_x);
        }
        if bound_y <= min_y {
            panic!("Bound y is {}, but min y is {}", bound_y, min_y);
        }
        Self {
            min_x,
            min_y,
            width: bound_x - min_x,
            height: bound_y - min_y,
        }
    }

    /// Gets the minimum x-coordinate of this region. This is the x-coordinate of
    /// the left-most pixel(s) of this region.
    pub fn get_min_x(&self) -> u32 {
        self.min_x
    }

    /// Gets the minimum y-coordinate of this region. This is the y-coordinate of
    /// the top (closest to the top of the screen) pixel(s) of this region.
    pub fn get_min_y(&self) -> u32 {
        self.min_y
    }

    /// Gets the maximum x-coordinate of this region. This is the x-coordinate of
    /// the right-most pixel(s) that is within this region.
    pub fn get_max_x(&self) -> u32 {
        self.min_x + self.width - 1
    }

    /// Gets the maximum y-coordinate of this region. This is the y-coordinate of
    /// the bottom (closest to the bottom of the screen) pixel(s) of this region.
    pub fn get_max_y(&self) -> u32 {
        self.min_y + self.height - 1
    }

    /// Gets the bound x-coordinate of this region. This is the x-coordinate of
    /// the left-most pixel(s) that are on the right of this render region. This is
    /// always equal to 1 + the *maximum* x-coordinate.
    pub fn get_bound_x(&self) -> u32 {
        self.min_x + self.width
    }

    /// Gets the bound y-coordinate of this region. This is the y-coordinate of
    /// the top-most pixel(s) that are below this render region. This is always equal
    /// to 1 + the *maximum* y-coordinate.
    pub fn get_bound_y(&self) -> u32 {
        self.min_y + self.height
    }

    /// Gets the width of this region, in pixels
    pub fn get_width(&self) -> u32 {
        self.width
    }

    /// Gets the height of this region, in pixels
    pub fn get_height(&self) -> u32 {
        self.height
    }

    /// Gets the aspect ratio of this region. This is just the width of this region
    /// divided by the height.
    pub fn get_aspect_ratio(&self) -> f32 {
        self.get_width() as f32 / self.get_height() as f32
    }

    /// Creates a child/sub region within this region with the given *relative*
    /// coordinates within this region.
    ///
    /// As example, using (0.0, 0.0, 1.0, 1.0) would return a copy of this region
    /// and using (0.0, 0.0, 0.5, 0.5) would return the bottom-left quarter of this
    /// region. If the area of the returned region would be 0, this method will return
    /// `None` instead. See the Examples for details.
    ///
    /// ### Examples
    /// ```
    /// use knukki::RenderRegion;
    ///
    /// let region = RenderRegion::between(20, 20, 30, 30);
    /// assert_eq!(Some(region), region.child_region(0.0, 0.0, 1.0, 1.0));
    /// assert_eq!(
    ///     Some(RenderRegion::between(20, 20, 25, 25)),
    ///     region.child_region(0.0, 0.0, 0.5, 0.5)
    /// );
    /// assert!(region.child_region(0.0, 0.0, 0.001, 0.001).is_none());
    /// ```
    pub fn child_region(
        &self,
        relative_min_x: f32,
        relative_min_y: f32,
        relative_max_x: f32,
        relative_max_y: f32,
    ) -> Option<Self> {

        let min_x = self.get_min_x() + (self.get_width() as f32 * relative_min_x).round() as u32;
        let min_y = self.get_min_y() + (self.get_height() as f32 * relative_min_y).round() as u32;

        let bound_x = self.get_min_x() + (self.get_width() as f32 * relative_max_x).round() as u32;
        let bound_y = self.get_min_y() + (self.get_height() as f32 * relative_max_y).round() as u32;

        if bound_x > min_x && bound_y > min_y {
            Some(Self::between(min_x, min_y, bound_x, bound_y))
        } else {
            None
        }
    }

    /// Computes the intersection of this region with the other region. That is, a new `RenderRegion`
    /// that covers the region where this region intersects/overlaps the other region. If this
    /// region doesn't have any overlap with the other region, this method returns `None`.
    ///
    /// # Examples
    /// ```
    /// use knukki::RenderRegion;
    ///
    /// // Simple case: the left region has some overlap with the right region
    /// let left = RenderRegion::between(0, 0, 40, 10);
    /// let right = RenderRegion::between(30, 0, 60, 10);
    /// let intersection = Some(RenderRegion::between(30, 0, 40, 10));
    /// assert_eq!(intersection, left.intersection(right));
    /// // The intersection is symmetric
    /// assert_eq!(intersection, right.intersection(left));
    ///
    /// // This one has no intersection with the left region
    /// let far_right = RenderRegion::between(100, 0, 200, 10);
    /// assert!(left.intersection(far_right).is_none());
    /// ```
    pub fn intersection(&self, other: Self) -> Option<Self> {
        let min_x = self.get_min_x().max(other.get_min_x());
        let min_y = self.get_min_y().max(other.get_min_y());
        let max_x = self.get_max_x().min(other.get_max_x());
        let max_y = self.get_max_y().min(other.get_max_y());

        if min_x <= max_x && min_y <= max_y {
            Some(Self::between(min_x, min_y, max_x + 1, max_y + 1))
        } else {
            None
        }
    }

    /// Sets the viewport of the given golem `Context` to this render region.
    #[cfg(feature = "golem_rendering")]
    pub fn set_viewport(&self, golem: &golem::Context) {
        golem.set_viewport(
            self.get_min_x(),
            self.get_min_y(),
            self.get_width(),
            self.get_height(),
        );
    }

    #[cfg(feature = "golem_rendering")]
    pub fn set_scissor(&self, golem: &golem::Context) {
        golem.set_scissor(
            self.get_min_x(),
            self.get_min_y(),
            self.get_width(),
            self.get_height()
        );
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn test_with_size() {
        let region = RenderRegion::with_size(10, 50, 20, 50);
        assert_eq!(10, region.get_min_x());
        assert_eq!(50, region.get_min_y());
        assert_eq!(20, region.get_width());
        assert_eq!(50, region.get_height());
        assert_eq!(29, region.get_max_x());
        assert_eq!(99, region.get_max_y());
        assert_eq!(30, region.get_bound_x());
        assert_eq!(100, region.get_bound_y());
    }

    #[test]
    fn test_between() {
        let region = RenderRegion::between(100, 80, 200, 150);
        assert_eq!(100, region.get_min_x());
        assert_eq!(80, region.get_min_y());
        assert_eq!(100, region.get_width());
        assert_eq!(70, region.get_height());
        assert_eq!(199, region.get_max_x());
        assert_eq!(149, region.get_max_y());
        assert_eq!(200, region.get_bound_x());
        assert_eq!(150, region.get_bound_y());
    }

    #[test]
    fn test_child_region() {
        let parent = RenderRegion::between(200, 500, 300, 600);
        assert_eq!(
            Some(RenderRegion::between(230, 530, 270, 570)),
            parent.child_region(0.3, 0.3, 0.7, 0.7)
        );
        assert_eq!(
            Some(RenderRegion::between(200, 500, 250, 550)),
            parent.child_region(0.0, 0.0, 0.5, 0.5)
        );
        assert_eq!(
            Some(RenderRegion::between(200, 550, 250, 600)),
            parent.child_region(0.0, 0.5, 0.5, 1.0)
        );
        assert_eq!(Some(parent), parent.child_region(0.0, 0.0, 1.0, 1.0));

        let mini_region = RenderRegion::with_size(100, 200, 1, 1);
        assert_eq!(Some(mini_region), mini_region.child_region(0.45, 0.45, 0.55, 0.55));
        assert_eq!(Some(mini_region), mini_region.child_region(0.0, 0.0, 1.0, 1.0));
        assert!(mini_region.child_region(0.1, 0.1, 0.4, 0.4).is_none());
    }

    #[test]
    fn test_intersection() {
        let region1 = RenderRegion::between(0, 0, 20, 20);
        let region_above = RenderRegion::between(0, 20, 20, 40);
        assert!(region1.intersection(region_above).is_none());

        let region_corner = RenderRegion::between(15, 16, 30, 35);
        assert_eq!(
            Some(RenderRegion::between(15, 16, 20, 20)),
            region1.intersection(region_corner)
        );

        let region_inner = RenderRegion::between(5, 6, 7, 8);
        assert_eq!(Some(region_inner), region1.intersection(region_inner));
        assert_eq!(Some(region_inner), region_inner.intersection(region1));

        let region_far = RenderRegion::with_size(100, 200, 300, 400);
        assert!(region1.intersection(region_far).is_none());

        let region_mini = RenderRegion::with_size(19, 19, 5, 5);
        assert_eq!(Some(RenderRegion::with_size(19, 19, 1, 1)), region1.intersection(region_mini));
    }
}
