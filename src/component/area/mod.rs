mod rectangle;

pub use rectangle::*;

use std::fmt::Debug;

/// Represents a part of the domain of a `Component`. The trait has an `is_inside`
/// method that decides whether a given point lies within this area, or not. 
/// This trait is used to let `Component`s tell which part of their domain they
/// are using at the moment.
/// 
/// ### Coordinate definitions
/// An x-coordinate of 0.0 indicates the left border of the component domain and
/// an x-coordinate of 1.0 indicates the right border of the component domain. 
/// Similarly, a y-coordinate of 0.0 indicates the bottom border of the component
/// domain and a y-coordinate of 1.0 indicates the top border of the component.
/// 
/// ### Implementations
/// The simplest implementation of this trait is `RectangleComponentArea`. I am
/// planning to add more implementations in the future. You can also create
/// your own implementations to define more complex shapes.
pub trait ComponentArea : Debug {

    /// Checks if the point (x, y) is inside this area and returns true if
    /// (and only if) so
    fn is_inside(&self, x: f32, y: f32) -> bool;

    /// Clones this component area. This method should normally return a new
    /// `ComponentArea` of the same struct as self. Due to Rust rules, this
    /// trait can't simply require implementing structs to implement `Clone`.
    fn clone(&self) -> Box<dyn ComponentArea>;

    /// Gets the left bound of this area. The is_inside method should return
    /// false for any point that is on the left of the left bound (whose
    /// x-coordinate is smaller than the result of this method).
    fn get_left(&self) -> f32;

    /// Gets the bottom bound of this area. The is_inside method should return
    /// false for any point that is below the bottom bound (whose y-coordinate
    /// is smaller than the result of this method).
    fn get_bottom(&self) -> f32;

    /// Gets the right bound of this area. The is_inside method should return
    /// false for any point that is on the right of the right bound (whose
    /// x-coordinate is larger than the result of this method).
    fn get_right(&self) -> f32;

    /// Gets the top bound of this area. The is_inside method should return
    /// false for any point that is above the right bound (whose
    /// y-coordinate is larger than the result of this method).
    fn get_top(&self) -> f32;
}
