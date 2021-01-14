use crate::Point;

mod composite;
mod line_intersection;
mod rectangle;
mod transformed;
mod oval;

pub use composite::*;
pub use line_intersection::*;
pub use rectangle::*;
pub use transformed::*;
pub use oval::*;

/// Represents a part of the domain of a `Component` and is used to indicate in
/// which part of its domain a component has actually drawn something.
///
/// ### Methods
/// The trait has an `is_inside` method that decides whether a given point lies
/// within this region, or not. Furthermore, this trait has methods to get the
/// `left_bound`, `right_bound`, `bottom_bound`, and `top_bound` of the region,
/// which should always be fast. For convenience, it also has an `is_within_bounds`
/// method that simply checks if a given point is within the left, right, bottom,
/// and top bounds (which is thus also quick).
///
/// ### Coordinate definitions
/// An x-coordinate of 0.0 indicates the left border of the component domain and
/// an x-coordinate of 1.0 indicates the right border of the component domain.
/// Similarly, a y-coordinate of 0.0 indicates the bottom border of the component
/// domain and a y-coordinate of 1.0 indicates the top border of the component.
///
/// ### Implementations
/// The simplest implementation of this trait are `RectangularDrawnRegion` and
/// `OvalDrawnRegion`. There is also the `CompositeDrawnRegion`, which can be used
/// to construct reasonably complex shapes by combining multiple other regions.
/// I am planning to add more implementations in the future. You can also create your
/// own implementations to define more complex shapes.
pub trait DrawnRegion {
    /// Checks if `point` is inside this region and returns true if (and only if) so
    fn is_inside(&self, point: Point) -> bool;

    /// Clones this drawn region. This method should normally return a new
    /// `DrawnRegion` of the same struct as self. Due to Rust rules, this
    /// trait can't simply require implementing structs to implement `Clone`.
    fn clone(&self) -> Box<dyn DrawnRegion>;

    /// Gets the left bound of this region. The `is_inside` method should return
    /// false for any point that is on the left of the left bound (whose
    /// x-coordinate is smaller than the result of this method).
    fn get_left(&self) -> f32;

    /// Gets the bottom bound of this region. The `is_inside` method should return
    /// false for any point that is below the bottom bound (whose y-coordinate
    /// is smaller than the result of this method).
    fn get_bottom(&self) -> f32;

    /// Gets the right bound of this region. The `is_inside` method should return
    /// false for any point that is on the right of the right bound (whose
    /// x-coordinate is larger than the result of this method).
    fn get_right(&self) -> f32;

    /// Gets the top bound of this region. The `is_inside` method should return
    /// false for any point that is above the right bound (whose
    /// y-coordinate is larger than the result of this method).
    fn get_top(&self) -> f32;

    /// Checks if the `point` is within the *bounds* of this `DrawnRegion`
    /// (thus whether `get_left()` <= `point.get_x()` <= `get_right()` and `get_bottom()` <=
    /// `point.get_y()` <= `get_top()`.
    ///
    /// This method should always be quick, no matter how complex this `DrawnRegion`
    /// is. Also, if this method returns `false`, `point` *can not* be
    /// *inside* this region. But if this method returns `true`, the possibly
    /// expensive `is_inside` method will have to be used to determine the final
    /// outcome.
    fn is_within_bounds(&self, point: Point) -> bool {
        point.get_x() >= self.get_left()
            && point.get_x() <= self.get_right()
            && point.get_y() >= self.get_bottom()
            && point.get_y() <= self.get_top()
    }

    /// Gets the width of this region. This is simply the result of subtracting
    /// the left bound from the right bound.
    fn get_width(&self) -> f32 {
        self.get_right() - self.get_left()
    }

    /// Gets the height of this region. This is simply the result of subtracting
    /// the bottom bound from the top bound.
    fn get_height(&self) -> f32 {
        self.get_top() - self.get_bottom()
    }

    /// Finds (or computes) the `LineIntersection` for the line(section) that starts at
    /// `from` and ends at `to`. See the documentation of `LineIntersection` for more information.
    fn find_line_intersection(&self, from: Point, to: Point) -> LineIntersection;
}
