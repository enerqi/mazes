// we need to compile and link to our library crate
// by default the `name = ???` of the [lib] is the same as the package name, so we extern crate to `mazes`

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate mazes;

use std::env;

use mazes::squaregrid::SquareGrid;
use mazes::generators;

fn main() {

    let grid_size = env::args().nth(1).map_or(20, |n_str| {
        n_str.parse::<u16>()
             .ok()
             .expect("Command Line Arg[1] (the grid size) should be a positive number")
    });

    let mut sg = SquareGrid::<u16>::new(grid_size);
    generators::binary_tree(&mut sg);
    println!("{}", sg);

    println!("");
    println!("");
    println!("");

    let mut sg_2 = SquareGrid::<u16>::new(grid_size);
    generators::sidewinder(&mut sg_2);
    println!("{}", sg_2);
}
