// we need to compile and link to our library crate
// by default the `name = ???` of the [lib] is the same as the package name, so we extern crate to `mazes`
extern crate mazes;

use mazes::grid;
use mazes::squaregrid::SquareGrid;

fn main() {
    println!("Hello, world!");
    let g = grid::Grid::new(5);

    let sg = SquareGrid::new(10*10);
}
