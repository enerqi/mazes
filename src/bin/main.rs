// we need to compile and link to our library crate
// by default the `name = ???` of the [lib] is the same as the package name, so we extern crate to `mazes`

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate mazes;

use mazes::squaregrid::SquareGrid;
use mazes::binary_tree_maze;

fn main() {
    let mut sg = SquareGrid::<u8>::new(10);
    binary_tree_maze::apply(&mut sg);
}
