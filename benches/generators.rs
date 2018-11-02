#![feature(test)]

use mazes::generators;
use mazes::grids::medium_rect_grid;
use mazes::units::{ColumnLength, RowLength};

use test::Bencher;

#[bench]
fn bench_binary_maze_32_u16(b: &mut Bencher) {

    let mut g = medium_rect_grid(RowLength(32), ColumnLength(32)).unwrap();

    b.iter(|| generators::binary_tree(&mut g));
}

#[bench]
fn bench_sidewinder_maze_32_u16(b: &mut Bencher) {

    let mut g = medium_rect_grid(RowLength(32), ColumnLength(32)).unwrap();

    b.iter(|| generators::sidewinder(&mut g));
}

#[bench]
fn bench_aldous_broder_maze_32_u16(b: &mut Bencher) {

    let mut g = medium_rect_grid(RowLength(32), ColumnLength(32)).unwrap();

    b.iter(|| generators::aldous_broder(&mut g, None));
}

#[bench]
fn bench_wilson_maze_32_u16(b: &mut Bencher) {

    let mut g = medium_rect_grid(RowLength(32), ColumnLength(32)).unwrap();

    b.iter(|| generators::wilson(&mut g, None));
}

#[bench]
fn bench_hunt_and_kill_maze_32_u16(b: &mut Bencher) {

    let mut g = medium_rect_grid(RowLength(32), ColumnLength(32)).unwrap();

    b.iter(|| generators::hunt_and_kill(&mut g, None));
}


#[bench]
fn bench_recursive_backtracker_maze_32_u16(b: &mut Bencher) {

    let mut g = medium_rect_grid(RowLength(32), ColumnLength(32)).unwrap();

    b.iter(|| generators::recursive_backtracker(&mut g, None));
}
