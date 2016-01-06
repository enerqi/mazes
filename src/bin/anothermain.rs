
// we need to compile and link to our library crate
// by default the `name = ???` of the [lib] is the same as the package name, so we extern crate to `mazes`
extern crate mazes;

use mazes::grid;

fn main() {

    println!("Another World!");

    let g = grid::Grid::new(5);

}
