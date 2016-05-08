#![feature(test)]

extern crate mazes;
extern crate test;

use mazes::squaregrid::SquareGrid;
use mazes::generators;
use test::Bencher;

#[bench]
fn bench_binary_maze_32_u16(b: &mut Bencher) {

    let mut g = SquareGrid::<u16>::new(32);

    b.iter(|| generators::binary_tree(&mut g));
}

#[bench]
fn bench_binary_maze_500(b: &mut Bencher) {

    let mut g = SquareGrid::<u32>::new(500);

    b.iter(|| generators::binary_tree(&mut g));
}

#[bench]
fn bench_sidewinder_maze_32_u16(b: &mut Bencher) {

    let mut g = SquareGrid::<u16>::new(32);

    b.iter(|| generators::sidewinder(&mut g));
}

#[bench]
fn bench_sidewinder_maze_500(b: &mut Bencher) {

    let mut g = SquareGrid::<u32>::new(500);

    b.iter(|| generators::sidewinder(&mut g));
}

#[bench]
fn bench_aldous_broder_maze_32_u16(b: &mut Bencher) {

    let mut g = SquareGrid::<u16>::new(32);

    b.iter(|| generators::aldous_broder(&mut g));
}

#[bench]
fn bench_wilson_maze_32_u16(b: &mut Bencher) {

    let mut g = SquareGrid::<u16>::new(32);

    b.iter(|| generators::wilson(&mut g));
}

#[bench]
fn bench_hunt_and_kill_maze_32_u16(b: &mut Bencher) {

    let mut g = SquareGrid::<u16>::new(32);

    b.iter(|| generators::hunt_and_kill(&mut g));
}
