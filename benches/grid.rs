#![feature(test)]

extern crate mazes;
extern crate test;

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
