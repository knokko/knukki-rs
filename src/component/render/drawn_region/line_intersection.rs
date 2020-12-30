use crate::Point;
use crate::LineIntersection::{FullyInside, FullyOutside};

/// This enum is the result of `DrawnRegion.findLineIntersection`. It indicates how a line(segment)
/// from a given starting point to a given ending point intersects a `DrawnRegion`: does the line
/// start outside or inside? does it end outside or inside? does it cross the region?
///
/// See the documentation of the individual enum options for more information.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LineIntersection {
    /// Both the starting point and the ending point of the line are inside the drawn region.
    FullyInside,
    /// Both the starting point and the ending point of the line are outside the drawn region, and
    /// the line does *not* cross the drawn region at all.
    FullyOutside,
    /// The starting point of the line is outside the drawn region, but the ending point is inside.
    /// The `point` is the (first) point where the line intersects the drawn region. (In
    /// complex drawn regions, there could be more than 1 intersection, in which case the one
    /// closest to the starting point will be given.)
    Enters{ point: Point },
    /// The starting point of the line is inside the drawn region, but the ending point is outside.
    /// The `point` is the (last) point where the line intersects the drawn region. (In
    /// complex drawn regions, there could be more than 1 intersection, in which case the one
    /// closest to the ending point will be given.)
    Exits{ point: Point },
    /// Both the starting point and the ending point of the line are outside the drawn region, but
    /// the line *does* intersect the drawn region. The first intersection is given by `entrance`
    /// and the last intersection is given by `exit`.
    Crosses{ entrance: Point, exit: Point }
}

impl LineIntersection {
    pub(crate) fn nearly_equal(&self, other: LineIntersection) -> bool {
        return match self {
            Self::FullyInside => other == FullyInside,
            Self::FullyOutside => other == FullyOutside,
            Self::Enters { point } => {
                if let Self::Enters { point: other_point} = other {
                    point.nearly_equal(other_point)
                } else {
                    false
                }
            },
            Self::Exits { point } => {
                if let Self::Exits { point: other_point} = other {
                    point.nearly_equal(other_point)
                } else {
                    false
                }
            }, Self::Crosses { entrance, exit } => {
                if let Self::Crosses { entrance: other_entrance, exit: other_exit } = other {
                    entrance.nearly_equal(other_entrance) && exit.nearly_equal(other_exit)
                } else {
                    false
                }
            }
        }
    }
}