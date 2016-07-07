//! **mazes** is a maze generation, visualisation and route finding library.

// TODO infrastructure:
// - public docs
// - deny dead_code, missing_docs
// - Variable GridCoordinate size (+ grid dimension size)

#![allow(dead_code, missing_docs)]
#![warn(variant_size_differences)]
#![deny(missing_debug_implementations, missing_copy_implementations,
        trivial_numeric_casts,
        unsafe_code,
        unused_extern_crates, unused_import_braces, unused_qualifications)]
#![cfg_attr(not(test), deny(trivial_casts))] // quickcheck test requirement

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", allow(similar_names))]

extern crate bit_set;
extern crate fnv;
extern crate image;
extern crate itertools;
extern crate num;
extern crate petgraph;
extern crate rand;
extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;
extern crate smallvec;

#[cfg(test)]
extern crate quickcheck;

pub mod cells;
pub mod generators;
pub mod grid;
pub mod grid_dimensions;
pub mod grid_displays;
pub mod grid_iterators;
pub mod grid_positions;
pub mod grid_traits;
pub mod masks;
pub mod pathing;
pub mod renderers;
pub mod units;
mod sdl;
mod utils;
