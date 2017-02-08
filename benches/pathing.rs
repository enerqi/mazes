#![feature(test)]

extern crate mazes;
extern crate test;


use mazes::cells::{Cartesian2DCoordinate, SquareCell};
use mazes::generators;
use mazes::grids::large_rect_grid;
use mazes::pathing;
use mazes::units::{ColumnLength, RowLength};
use test::Bencher;

type SquareCellDistances = pathing::Distances<SquareCell, u32>;

#[bench]
fn bench_distances(b: &mut Bencher) {

    let mut g = large_rect_grid(RowLength(350), ColumnLength(350)).unwrap();
    generators::recursive_backtracker(&mut g, None);
    let start_coord = Cartesian2DCoordinate::new(250, 250);

    b.iter(|| SquareCellDistances::new(&g, start_coord));
}

#[bench]
fn bench_furthest_points(b: &mut Bencher) {

    let mut g = large_rect_grid(RowLength(350), ColumnLength(350)).unwrap();
    generators::recursive_backtracker(&mut g, None);
    let start_coord = Cartesian2DCoordinate::new(250, 250);
    let distances = SquareCellDistances::new(&g, start_coord).unwrap();

    b.iter(|| distances.furthest_points_on_grid());
}

#[bench]
fn bench_shortest_path(b: &mut Bencher) {

    let mut g = large_rect_grid(RowLength(350), ColumnLength(350)).unwrap();
    generators::recursive_backtracker(&mut g, None);
    let start_coord = Cartesian2DCoordinate::new(250, 250);
    let distances = SquareCellDistances::new(&g, start_coord).unwrap();
    let end_coord = Cartesian2DCoordinate::new(0, 0);

    b.iter(|| pathing::shortest_path(&g, &distances, end_coord));
}
