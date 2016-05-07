#![feature(test)]

extern crate mazes;
extern crate test;

use mazes::squaregrid::SquareGrid;
use mazes::generators;
use test::Bencher;

#[bench]
fn bench_binary_maze_11_u8(b: &mut Bencher) {

    let mut g = SquareGrid::<u8>::new(11);

    b.iter(|| generators::binary_tree(&mut g));
}

#[bench]
fn bench_binary_maze_11_u16(b: &mut Bencher) {

    let mut g = SquareGrid::<u16>::new(11);

    b.iter(|| generators::binary_tree(&mut g));
}

#[bench]
fn bench_binary_maze_11_u32(b: &mut Bencher) {

    let mut g = SquareGrid::<u32>::new(11);

    b.iter(|| generators::binary_tree(&mut g));
}

#[bench]
fn bench_binary_maze_128_u16(b: &mut Bencher) {

    let mut g = SquareGrid::<u16>::new(128);

    b.iter(|| generators::binary_tree(&mut g));
}

#[bench]
fn bench_binary_maze_128_u32(b: &mut Bencher) {

    let mut g = SquareGrid::<u32>::new(128);

    b.iter(|| generators::binary_tree(&mut g));
}

#[bench]
fn bench_binary_maze_500(b: &mut Bencher) {

    let mut g = SquareGrid::<u32>::new(500);

    b.iter(|| generators::binary_tree(&mut g));
}

#[bench]
fn bench_sidewinder_maze_128_u16(b: &mut Bencher) {

    let mut g = SquareGrid::<u16>::new(128);

    b.iter(|| generators::sidewinder(&mut g));
}

#[bench]
fn bench_sidewinder_maze_128_u32(b: &mut Bencher) {

    let mut g = SquareGrid::<u32>::new(128);

    b.iter(|| generators::sidewinder(&mut g));
}

#[bench]
fn bench_sidewinder_maze_500(b: &mut Bencher) {

    let mut g = SquareGrid::<u32>::new(500);

    b.iter(|| generators::sidewinder(&mut g));
}

#[bench]
fn bench_random_walk_maze_128_u16(b: &mut Bencher) {

    let mut g = SquareGrid::<u16>::new(128);

    b.iter(|| generators::random_walk(&mut g));
}

#[bench]
fn bench_random_walk_maze_128_u32(b: &mut Bencher) {

    let mut g = SquareGrid::<u32>::new(128);

    b.iter(|| generators::random_walk(&mut g));
}

#[bench]
fn bench_random_walk_maze_250(b: &mut Bencher) {

    let mut g = SquareGrid::<u32>::new(250);

    b.iter(|| generators::random_walk(&mut g));
}
