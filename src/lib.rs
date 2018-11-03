//! **mazes** is a maze generation, visualisation and route finding library.

// TODO infrastructure:
// - public docs / tutorial / examples

pub mod cells;
pub mod generators;
pub mod grid;
pub mod grid_coordinates;
pub mod grid_dimensions;
pub mod grid_displays;
pub mod grid_iterators;
pub mod grid_traits;
pub mod grids;
pub mod masks;
pub mod pathing;
pub mod renderers;
pub mod units;
mod sdl;
mod utils;
