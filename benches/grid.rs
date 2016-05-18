#![feature(test)]

extern crate mazes;
extern crate test;

use mazes::squaregrid;
use mazes::squaregrid::SquareGrid;
use test::Bencher;

#[bench]
fn bench_maze_11_u8(b: &mut Bencher) {

    b.iter(|| SquareGrid::<u8>::new(11));
}

#[bench]
fn bench_maze_11_u16(b: &mut Bencher) {

    b.iter(|| SquareGrid::<u16>::new(11));
}

#[bench]
fn bench_maze_11_u32(b: &mut Bencher) {

    b.iter(|| SquareGrid::<u32>::new(11));
}

#[bench]
fn bench_maze_128_u16(b: &mut Bencher) {

    b.iter(|| SquareGrid::<u16>::new(128));
}

#[bench]
fn bench_maze_128_u32(b: &mut Bencher) {

    b.iter(|| SquareGrid::<u32>::new(128));
}

#[bench]
fn bench_maze_500(b: &mut Bencher) {

    b.iter(|| SquareGrid::<u32>::new(500));
}

#[bench]
fn bench_index_to_gridcoordinate(b: &mut Bencher) {
    let g = SquareGrid::<u32>::new(11);
    let dim = g.dimension();
    b.iter(|| squaregrid::index_to_grid_coordinate(dim, 93));
}

#[bench]
fn bench_neighbours_corner_of_grid(b: &mut Bencher) {
    let g = SquareGrid::<u32>::new(11);
    let corner = squaregrid::GridCoordinate::new(0, 0);
    b.iter(|| g.neighbours(corner));
}

#[bench]
fn bench_neighbours_middle_of_grid(b: &mut Bencher) {
    let g = SquareGrid::<u32>::new(11);
    let mid = squaregrid::GridCoordinate::new(5, 5);
    b.iter(|| g.neighbours(mid));
}
