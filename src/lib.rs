//! **mazes** is a maze generation, visualisation and route finding library.

// TODO infrastructure:
// - public docs

#![allow(dead_code, missing_docs)]
#![warn(variant_size_differences)]
#![deny(missing_debug_implementations, missing_copy_implementations,
        trivial_numeric_casts,
        unsafe_code,
        unused_extern_crates, unused_import_braces, unused_qualifications)]
#![cfg_attr(not(test), deny(trivial_casts))] // quickcheck test requirement

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate fnv;
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

pub mod generators;
pub mod pathing;
pub mod renderers;
pub mod squaregrid;
mod sdl;
mod utils;
