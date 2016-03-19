//! **mazes** is a maze generation, visualisation and route finding library.

// TODO infrastructure:
// - public docs
// - quickcheck experiments
// - bench experiments (nightly)

#![allow(dead_code, missing_docs)]
#![warn(variant_size_differences)]
#![deny(missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unused_extern_crates, unused_import_braces, unused_qualifications)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate fnv;
extern crate num;
extern crate petgraph;
extern crate rand;
extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;
extern crate smallvec;

#[cfg(test)]
extern crate itertools;

pub mod generators;
pub mod pathing;
pub mod renderers;
pub mod squaregrid;
mod sdl;
