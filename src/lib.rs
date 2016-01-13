// This file is our crate root

#![allow(dead_code)]
#![allow(dead_code)]

// Note we could put this in e.g. grid.rs, but then when we use it
// from that sub-module we would `use self::rand::Rng` instead of
// `rand::Rng`, which means crate_root::rand::Rng;
extern crate petgraph;
extern crate rand;

// look for one of
// (1) src/grid.rs
// (2) src/grid/mod.rs (preferred when we have sub-modules)
// The contents of these files are now in the respective modules
// This is a declaration
// e.g. maze::cell = cell.rs
// So a module exists in the *context* of the crate root module
// A crate = module that can be linked to and unit of compilation for the compiler
pub mod squaregrid;

// All public items within a crate gets a symbol exposed to the linker
//
// Multi file crates are like a giant file in disguise (cf #include), but the scoping
// rules create neat little boxes - each module gets its own symbol table even when
// in the same file

