// we need to compile and link to our library crate
// by default the `name = ???` of the [lib] is the same as the package name, so we extern crate to `mazes`

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate mazes;

use mazes::squaregrid::SquareGrid;
use mazes::generators;

fn main() {
    let mut sg = SquareGrid::<u16>::new(20);
    generators::binary_tree(&mut sg);
    println!("{}", sg);

    println!("");
    println!("");
    println!("");

    let mut sg_2 = SquareGrid::<u16>::new(50);
    generators::sidewinder(&mut sg_2);
    println!("{}", sg_2);
}
