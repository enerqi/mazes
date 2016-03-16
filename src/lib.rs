#![allow(dead_code)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate num;
extern crate petgraph;
extern crate rand;
extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;

#[cfg(test)]
extern crate itertools;

pub mod generators;
pub mod pathing;
pub mod renderers;
pub mod squaregrid;
mod sdl;
