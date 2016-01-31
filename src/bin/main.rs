// we need to compile and link to our library crate
// by default the `name = ???` of the [lib] is the same as the package name, so we extern crate to `mazes`

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate mazes;

use mazes::squaregrid::SquareGrid;

fn main() {
    let sg = SquareGrid::<u8>::new(10);
}
