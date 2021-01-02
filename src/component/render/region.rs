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
    pub fn with_size(min_x: u32, min_y: u32, width: u32, height: u32) -> Self {
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
    /// ### Panic
    /// This function will panic if *bound_x* < *min_x* or *bound_y* < *min_y*
    pub fn between(min_x: u32, min_y: u32, bound_x: u32, bound_y: u32) -> Self {
        if bound_x < min_x {
            panic!("Bound x is {}, but min x is {}", bound_x, min_x);
        }
        if bound_y < min_y {
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
    /// region. See the Examples for details.
    ///
    /// ### Examples
    /// ```
    /// use knukki::RenderRegion;
    ///
    /// let region = RenderRegion::between(20, 20, 30, 30);
    /// assert_eq!(region, region.child_region(0.0, 0.0, 1.0, 1.0));
    /// assert_eq!(
    ///     RenderRegion::between(20, 20, 25, 25),
    ///     region.child_region(0.0, 0.0, 0.5, 0.5)
    /// );
    /// ```
    pub fn child_region(
        &self,
        relative_min_x: f32,
        relative_min_y: f32,
        relative_max_x: f32,
        relative_max_y: f32,
    ) -> Self {
        let relative_width = relative_max_x - relative_min_x;
        let relative_height = relative_max_y - relative_min_y;

        let width = (self.get_width() as f32 * relative_width).round() as u32;
        let height = (self.get_height() as f32 * relative_height).round() as u32;

        let min_x = self.get_min_x() + (self.get_width() as f32 * relative_min_x).round() as u32;
        let min_y = self.get_min_y() + (self.get_height() as f32 * relative_min_y).round() as u32;

        return Self::with_size(min_x, min_y, width, height);
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
            RenderRegion::between(230, 530, 270, 570),
            parent.child_region(0.3, 0.3, 0.7, 0.7)
        );
        assert_eq!(
            RenderRegion::between(200, 500, 250, 550),
            parent.child_region(0.0, 0.0, 0.5, 0.5)
        );
        assert_eq!(
            RenderRegion::between(200, 550, 250, 600),
            parent.child_region(0.0, 0.5, 0.5, 1.0)
        );
        assert_eq!(parent, parent.child_region(0.0, 0.0, 1.0, 1.0));
    }
}
