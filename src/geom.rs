//! Geometry types used for layour and rendering

/// Describes a rectangle in 2D space.
#[derive(Debug, Default, Clone, Copy)]
pub struct Rect {
    /// The x coordinate.
    pub x: usize,
    /// The y coordinate.
    pub y: usize,
    /// The width of the rectangle. Should be constructed from the `cols` argument received in the render callback.
    pub width: usize,
    /// The height of the rectangle. Should be constructed from the `rows` argument received in the render callback.
    pub height: usize,
}
