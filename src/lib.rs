#![allow(dead_code)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate petgraph;
extern crate rand;
extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;

#[cfg(test)]
extern crate itertools;

// look for one of
// (1) src/grid.rs
// (2) src/grid/mod.rs (preferred when we have sub-modules)
// The contents of these files are now in the respective modules
// This is a declaration
// e.g. maze::cell = cell.rs
// So a module exists in the *context* of the crate root module
// A crate = module that can be linked to and unit of compilation for the compiler
pub mod generators;
pub mod renderers;
pub mod squaregrid;
mod sdl;

// All public items within a crate gets a symbol exposed to the linker
//
// Multi file crates are like a giant file in disguise (cf #include), but the scoping
// rules create neat little boxes - each module gets its own symbol table even when
// in the same file
